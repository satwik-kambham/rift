use std::collections::HashMap;

use client::LSPClientHandle;

use crate::{
    actions::{perform_action, Action},
    buffer::instance::{Cursor, Language, Selection},
    state::EditorState,
};

pub mod client;
pub mod types;

pub fn handle_lsp_messages(
    state: &mut EditorState,
    lsp_handles: &mut HashMap<Language, LSPClientHandle>,
) {
    if state.buffer_idx.is_some() {
        let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
        if let Some(lsp_handle) = lsp_handles.get_mut(&buffer.language) {
            if let Some(message) = lsp_handle.recv_message_sync() {
                match message {
                    client::IncomingMessage::Response(response) => {
                        if response.error.is_some() {
                            tracing::error!(
                                "---Error: Message Id: {}\n\n{:#?}---\n",
                                response.id,
                                response.error.unwrap()
                            );
                        } else if lsp_handle.id_method[&response.id] == "textDocument/hover"
                            && response.result.is_some()
                        {
                            let message = response.result.unwrap()["contents"]["value"]
                                .as_str()
                                .unwrap()
                                .to_string();
                            state.info_modal.open(message);
                        } else if lsp_handle.id_method[&response.id] == "textDocument/completion"
                            && response.result.is_some()
                        {
                            let items = response.result.unwrap()["items"]
                                .as_array()
                                .unwrap()
                                .clone();
                            let mut completion_items = vec![];
                            for item in items {
                                completion_items.push(types::CompletionItem {
                                    label: item["label"].as_str().unwrap().to_owned(),
                                    edit: types::TextEdit {
                                        text: item["textEdit"]["newText"]
                                            .as_str()
                                            .unwrap()
                                            .to_owned(),
                                        range: Selection {
                                            cursor: Cursor {
                                                row: item["textEdit"]["range"]["end"]["line"]
                                                    .as_u64()
                                                    .unwrap()
                                                    as usize,
                                                column: item["textEdit"]["range"]["end"]
                                                    ["character"]
                                                    .as_u64()
                                                    .unwrap()
                                                    as usize,
                                            },
                                            mark: Cursor {
                                                row: item["textEdit"]["range"]["start"]["line"]
                                                    .as_u64()
                                                    .unwrap()
                                                    as usize,
                                                column: item["textEdit"]["range"]["start"]
                                                    ["character"]
                                                    .as_u64()
                                                    .unwrap()
                                                    as usize,
                                            },
                                        },
                                    },
                                });
                            }
                            state.completion_menu.open(completion_items);
                        } else if lsp_handle.id_method[&response.id] == "textDocument/formatting"
                            && response.result.is_some()
                        {
                            let edits = response.result.unwrap().as_array().unwrap().clone();
                            for edit in edits {
                                let text_edit = types::TextEdit {
                                    text: edit["newText"].as_str().unwrap().to_owned(),
                                    range: Selection {
                                        cursor: Cursor {
                                            row: edit["range"]["end"]["line"].as_u64().unwrap()
                                                as usize,
                                            column: edit["range"]["end"]["character"]
                                                .as_u64()
                                                .unwrap()
                                                as usize,
                                        },
                                        mark: Cursor {
                                            row: edit["range"]["start"]["line"].as_u64().unwrap()
                                                as usize,
                                            column: edit["range"]["start"]["character"]
                                                .as_u64()
                                                .unwrap()
                                                as usize,
                                        },
                                    },
                                };
                                perform_action(
                                    Action::DeleteText(text_edit.range),
                                    state,
                                    lsp_handles,
                                );
                                perform_action(
                                    Action::InsertText(text_edit.text, text_edit.range.mark),
                                    state,
                                    lsp_handles,
                                );
                            }
                        } else if lsp_handle.id_method[&response.id] == "textDocument/signatureHelp"
                            && response.result.is_some()
                        {
                            let label = response.result.unwrap()["signatures"]
                                .as_array()
                                .unwrap()
                                .first()
                                .unwrap()["label"]
                                .as_str()
                                .unwrap()
                                .to_string();
                            state.signature_information.content = label;
                        } else {
                            let message = format!(
                                "---Response to: {}({})\n\n{:#?}---\n",
                                lsp_handle.id_method[&response.id], response.id, response.result
                            );
                            tracing::info!("{}", message);
                        }
                    }
                    client::IncomingMessage::Notification(notification) => {
                        if notification.method == "textDocument/publishDiagnostics"
                            && notification.params.is_some()
                        {
                            let mut uri = std::path::absolute(
                                notification.params.as_ref().unwrap()["uri"]
                                    .as_str()
                                    .unwrap()
                                    .strip_prefix("file:")
                                    .unwrap()
                                    .trim_start_matches("\\"),
                            )
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string();
                            #[cfg(target_os = "windows")]
                            {
                                uri = uri.to_lowercase();
                            }

                            let mut diagnostics = types::PublishDiagnostics {
                                uri,
                                version: notification.params.as_ref().unwrap()["version"]
                                    .as_u64()
                                    .unwrap_or(0) as usize,
                                diagnostics: vec![],
                            };

                            for diagnostic in notification.params.as_ref().unwrap()["diagnostics"]
                                .as_array()
                                .unwrap()
                            {
                                diagnostics.diagnostics.push(types::Diagnostic {
                                    range: Selection {
                                        cursor: Cursor {
                                            row: diagnostic["range"]["end"]["line"]
                                                .as_u64()
                                                .unwrap()
                                                as usize,
                                            column: diagnostic["range"]["end"]["character"]
                                                .as_u64()
                                                .unwrap()
                                                as usize,
                                        },
                                        mark: Cursor {
                                            row: diagnostic["range"]["start"]["line"]
                                                .as_u64()
                                                .unwrap()
                                                as usize,
                                            column: diagnostic["range"]["start"]["character"]
                                                .as_u64()
                                                .unwrap()
                                                as usize,
                                        },
                                    },
                                    severity: match diagnostic["severity"].as_u64().unwrap_or(1) {
                                        1 => types::DiagnosticSeverity::Error,
                                        2 => types::DiagnosticSeverity::Warning,
                                        3 => types::DiagnosticSeverity::Information,
                                        4 => types::DiagnosticSeverity::Hint,
                                        _ => types::DiagnosticSeverity::Error,
                                    },
                                    code: diagnostic["code"].to_string(),
                                    source: diagnostic["source"].to_string(),
                                    message: diagnostic["message"].to_string(),
                                });
                            }
                            state
                                .diagnostics
                                .insert(diagnostics.uri.clone(), diagnostics);
                        } else {
                            let message = format!(
                                "---Notification: {}\n\n{:#?}---\n",
                                notification.method, notification.params
                            );
                            tracing::info!("{}", message);
                            state.diagnostics_overlay.content = message;
                        }
                    }
                }
            }
        }
    }
}
