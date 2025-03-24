use std::collections::HashMap;

use client::LSPClientHandle;

use crate::{
    actions::{perform_action, Action},
    buffer::instance::{Cursor, Language, Selection},
    state::EditorState,
};

pub mod client;
pub mod types;

pub fn parse_uri(uri: String) -> String {
    let uri = std::path::absolute(uri.strip_prefix("file:").unwrap().trim_start_matches("\\"))
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    #[cfg(target_os = "windows")]
    let uri = uri.to_lowercase();

    uri
}

pub fn parse_range(range: &serde_json::Value) -> Selection {
    Selection {
        cursor: Cursor {
            row: range["end"]["line"].as_u64().unwrap() as usize,
            column: range["end"]["character"].as_u64().unwrap() as usize,
        },
        mark: Cursor {
            row: range["start"]["line"].as_u64().unwrap() as usize,
            column: range["start"]["character"].as_u64().unwrap() as usize,
        },
    }
}

pub fn handle_lsp_messages(
    state: &mut EditorState,
    lsp_handles: &mut HashMap<Language, LSPClientHandle>,
) {
    if state.buffer_idx.is_some() {
        let (buffer, instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
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
                                .unwrap_or_default()
                                .to_string();
                            state.info_modal.open(message);
                        } else if lsp_handle.id_method[&response.id] == "textDocument/completion"
                            && response.result.is_some()
                        {
                            let items = if response.result.as_ref().unwrap()["items"].is_array() {
                                response.result.unwrap()["items"]
                                    .as_array()
                                    .unwrap()
                                    .clone()
                            } else {
                                response.result.unwrap().as_array().unwrap().clone()
                            };
                            let mut completion_items = vec![];
                            for item in items {
                                let label = item["label"].as_str().unwrap().to_owned();
                                if label.contains(&buffer.get_word_under_cursor(&instance.cursor)) {
                                    if item["textEdit"].is_object() {
                                        completion_items.push(types::CompletionItem {
                                            label,
                                            edit: types::TextEdit {
                                                text: item["textEdit"]["newText"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_owned(),
                                                range: parse_range(&item["textEdit"]["range"]),
                                            },
                                        });
                                    } else if item["insertText"].is_string() {
                                        completion_items.push(types::CompletionItem {
                                            label,
                                            edit: types::TextEdit {
                                                text: item["insertText"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_owned(),
                                                range: buffer
                                                    .get_word_range_under_cursor(&instance.cursor),
                                            },
                                        });
                                    } else {
                                        completion_items.push(types::CompletionItem {
                                            label,
                                            edit: types::TextEdit {
                                                text: item["label"].as_str().unwrap().to_owned(),
                                                range: Selection {
                                                    cursor: instance.cursor,
                                                    mark: instance.cursor,
                                                },
                                            },
                                        });
                                    }
                                }
                            }
                            state.completion_menu.open(completion_items);
                        } else if lsp_handle.id_method[&response.id] == "textDocument/formatting"
                            && response.result.is_some()
                        {
                            let edits = response.result.unwrap().as_array().unwrap().clone();
                            for edit in edits.iter().rev() {
                                let text_edit = types::TextEdit {
                                    text: edit["newText"].as_str().unwrap().to_owned(),
                                    range: parse_range(&edit["range"]),
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
                            if !response.result.as_ref().unwrap()["signatures"]
                                .as_array()
                                .unwrap()
                                .is_empty()
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
                            }
                        } else if lsp_handle.id_method[&response.id] == "textDocument/definition"
                            && response.result.is_some()
                        {
                            if response.result.as_ref().unwrap().is_array() {
                                let mut locations = vec![];
                                for location in response.result.unwrap().as_array().unwrap() {
                                    let uri =
                                        parse_uri(location["uri"].as_str().unwrap().to_string());
                                    let range = parse_range(&location["range"]);
                                    locations.push((uri, range));
                                }
                                perform_action(
                                    Action::LocationModal(locations),
                                    state,
                                    lsp_handles,
                                );
                            } else {
                                let location = response.result.unwrap();
                                let uri = parse_uri(location["uri"].as_str().unwrap().to_string());
                                let range = parse_range(&location["range"]);
                                perform_action(
                                    Action::CreateBufferFromFile(uri),
                                    state,
                                    lsp_handles,
                                );
                                perform_action(Action::Select(range), state, lsp_handles);
                            }
                        } else if lsp_handle.id_method[&response.id] == "textDocument/references"
                            && response.result.is_some()
                        {
                            let mut locations = vec![];
                            for location in response.result.unwrap().as_array().unwrap() {
                                let uri = parse_uri(location["uri"].as_str().unwrap().to_string());
                                let range = parse_range(&location["range"]);
                                locations.push((uri, range));
                            }
                            perform_action(Action::LocationModal(locations), state, lsp_handles);
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
                            let uri = parse_uri(
                                notification.params.as_ref().unwrap()["uri"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                            );

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
                                    range: parse_range(&diagnostic["range"]),
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
