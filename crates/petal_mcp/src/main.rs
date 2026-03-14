use std::path::PathBuf;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use clap::Parser;
use petal::NoteStore;
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use tracing_subscriber::{self, EnvFilter};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "petal-mcp", about = "MCP server for the petal note store")]
struct Cli {
    /// Path to the petal store directory (default: $HOME/petal)
    #[arg(long)]
    petal_path: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Tool parameter types
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct NoteIdParam {
    #[schemars(description = "UUID of the note")]
    note_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ReadAttachmentParam {
    #[schemars(description = "UUID of the note")]
    note_id: String,
    #[schemars(description = "Filename of the attachment")]
    filename: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateNoteParam {
    #[schemars(description = "Optional title for the new note")]
    title: Option<String>,
    #[schemars(description = "Text content for the initial text block")]
    content: String,
    #[schemars(description = "Optional label for the text block")]
    label: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct AddTextBlockParam {
    #[schemars(description = "UUID of the note to add a text block to")]
    note_id: String,
    #[schemars(description = "Text content for the block")]
    content: String,
    #[schemars(description = "Optional label for the text block")]
    label: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct UpdateTextBlockParam {
    #[schemars(description = "UUID of the note")]
    note_id: String,
    #[schemars(description = "Zero-based index of the text block to update")]
    block_index: usize,
    #[schemars(description = "New text content for the block")]
    content: String,
    #[schemars(description = "New label for the text block (null to clear)")]
    label: Option<String>,
}

// ---------------------------------------------------------------------------
// MCP Server
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct PetalServer {
    store_path: PathBuf,
    tool_router: ToolRouter<Self>,
}

impl PetalServer {
    fn store(&self) -> std::result::Result<NoteStore, String> {
        NoteStore::open(&self.store_path).map_err(|e| format!("failed to open note store: {e}"))
    }
}

fn parse_uuid(s: &str) -> std::result::Result<Uuid, String> {
    Uuid::parse_str(s).map_err(|e| format!("invalid UUID '{s}': {e}"))
}

#[tool_router]
impl PetalServer {
    fn new(store_path: PathBuf) -> Self {
        Self {
            store_path,
            tool_router: Self::tool_router(),
        }
    }

    // -- Read tools ---------------------------------------------------------

    #[tool(
        description = "List all notes in the petal store. Returns an array of note summaries with id, title, timestamps, and block count."
    )]
    fn list_notes(&self) -> String {
        let store = match self.store() {
            Ok(s) => s,
            Err(e) => return e,
        };
        match store.list_notes() {
            Ok(notes) => {
                let summaries: Vec<serde_json::Value> = notes
                    .iter()
                    .map(|n| {
                        serde_json::json!({
                            "id": n.id.to_string(),
                            "title": n.title,
                            "created_at": n.created_at.to_rfc3339(),
                            "updated_at": n.updated_at.to_rfc3339(),
                            "block_count": n.blocks.len(),
                        })
                    })
                    .collect();
                serde_json::to_string_pretty(&summaries).unwrap_or_default()
            }
            Err(e) => format!("error listing notes: {e}"),
        }
    }

    #[tool(description = "Get the full content of a note by its UUID, including all blocks.")]
    fn get_note(&self, Parameters(param): Parameters<NoteIdParam>) -> String {
        let store = match self.store() {
            Ok(s) => s,
            Err(e) => return e,
        };
        let id = match parse_uuid(&param.note_id) {
            Ok(id) => id,
            Err(e) => return e,
        };
        match store.get_note(id) {
            Ok(note) => serde_json::to_string_pretty(&note).unwrap_or_default(),
            Err(e) => format!("error getting note: {e}"),
        }
    }

    #[tool(description = "Read a binary attachment from a note. Returns base64-encoded data.")]
    fn read_attachment(&self, Parameters(param): Parameters<ReadAttachmentParam>) -> String {
        let store = match self.store() {
            Ok(s) => s,
            Err(e) => return e,
        };
        let id = match parse_uuid(&param.note_id) {
            Ok(id) => id,
            Err(e) => return e,
        };
        match store.read_attachment(id, &param.filename) {
            Ok(data) => BASE64.encode(&data),
            Err(e) => format!("error reading attachment: {e}"),
        }
    }

    // -- Write tools --------------------------------------------------------

    #[tool(
        description = "Create a new note with an initial text block. Returns the created note as JSON."
    )]
    fn create_note(&self, Parameters(param): Parameters<CreateNoteParam>) -> String {
        let store = match self.store() {
            Ok(s) => s,
            Err(e) => return e,
        };
        let mut note = match store.create_note() {
            Ok(n) => n,
            Err(e) => return format!("error creating note: {e}"),
        };
        note.title = param.title;
        note.blocks.push(petal::Block::Text {
            label: param.label,
            content: param.content,
        });
        match store.save_note(note) {
            Ok(saved) => serde_json::to_string_pretty(&saved).unwrap_or_default(),
            Err(e) => format!("error saving note: {e}"),
        }
    }

    #[tool(
        description = "Add a new text block to an existing note. Returns the updated note as JSON."
    )]
    fn add_text_block(&self, Parameters(param): Parameters<AddTextBlockParam>) -> String {
        let store = match self.store() {
            Ok(s) => s,
            Err(e) => return e,
        };
        let id = match parse_uuid(&param.note_id) {
            Ok(id) => id,
            Err(e) => return e,
        };
        let mut note = match store.get_note(id) {
            Ok(n) => n,
            Err(e) => return format!("error getting note: {e}"),
        };
        note.blocks.push(petal::Block::Text {
            label: param.label,
            content: param.content,
        });
        match store.save_note(note) {
            Ok(saved) => serde_json::to_string_pretty(&saved).unwrap_or_default(),
            Err(e) => format!("error saving note: {e}"),
        }
    }

    #[tool(
        description = "Update the content and label of an existing text block in a note. Returns the updated note as JSON."
    )]
    fn update_text_block(&self, Parameters(param): Parameters<UpdateTextBlockParam>) -> String {
        let store = match self.store() {
            Ok(s) => s,
            Err(e) => return e,
        };
        let id = match parse_uuid(&param.note_id) {
            Ok(id) => id,
            Err(e) => return e,
        };
        let mut note = match store.get_note(id) {
            Ok(n) => n,
            Err(e) => return format!("error getting note: {e}"),
        };
        if param.block_index >= note.blocks.len() {
            return format!(
                "block index {} out of range (note has {} blocks)",
                param.block_index,
                note.blocks.len()
            );
        }
        match &note.blocks[param.block_index] {
            petal::Block::Text { .. } => {
                note.blocks[param.block_index] = petal::Block::Text {
                    label: param.label,
                    content: param.content,
                };
            }
            petal::Block::Attachment { .. } => {
                return format!(
                    "block at index {} is an attachment, not a text block",
                    param.block_index
                );
            }
        }
        match store.save_note(note) {
            Ok(saved) => serde_json::to_string_pretty(&saved).unwrap_or_default(),
            Err(e) => format!("error saving note: {e}"),
        }
    }
}

#[tool_handler]
impl ServerHandler for PetalServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "petal-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                description: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Petal note store — read and write notes and text blocks".to_string(),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let cli = Cli::parse();
    let store_path = cli.petal_path.unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join("petal")
    });

    tracing::info!(path = %store_path.display(), "Starting petal MCP server");

    // Validate the store can be opened before serving
    NoteStore::open(&store_path)?;

    let server = PetalServer::new(store_path)
        .serve(rmcp::transport::stdio())
        .await
        .inspect_err(|e| tracing::error!("serving error: {:?}", e))?;

    server.waiting().await?;
    Ok(())
}
