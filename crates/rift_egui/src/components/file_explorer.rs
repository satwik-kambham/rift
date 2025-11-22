use std::collections::HashMap;

use egui::{Label, Popup, Sense};
use rift_core::{
    actions::{perform_action, Action},
    buffer::instance::Language,
    io::file_io::{self, FolderEntry},
    lsp::client::LSPClientHandle,
    state::EditorState,
};

pub struct FileExplorer {
    pub workspace_folder: String,
    pub entries: Vec<FolderEntry>,
}

impl FileExplorer {
    pub fn new() -> Self {
        Self {
            workspace_folder: String::new(),
            entries: vec![],
        }
    }

    pub fn set_entry(entries: &mut [FolderEntry], path: &str, children: Option<Vec<FolderEntry>>) {
        for entry in entries.iter_mut() {
            if entry.is_dir {
                if entry.path == path {
                    entry.children = children;
                    return;
                } else if path.starts_with(&entry.path) {
                    FileExplorer::set_entry(entry.children.as_mut().unwrap(), path, children);
                    return;
                }
            }
        }
    }

    pub fn update_entries(&mut self, path: Option<String>, clear: bool) {
        if let Some(path) = path {
            FileExplorer::set_entry(
                &mut self.entries,
                &path,
                if !clear {
                    Some(file_io::get_directory_entries(&path).unwrap())
                } else {
                    None
                },
            );
        } else {
            self.entries = file_io::get_directory_entries(&self.workspace_folder).unwrap();
        }
    }

    pub fn render(
        &mut self,
        entries: Vec<FolderEntry>,
        spacing: usize,
        ui: &mut egui::Ui,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) {
        for entry in entries.clone().iter() {
            if entry.is_dir {
                if entry.children.is_some() {
                    ui.horizontal(|ui| {
                        ui.label(" ".repeat(spacing));
                        ui.image(egui::include_image!("../../../../assets/Folder.svg"));
                        let response = ui.add(Label::new(&entry.name).sense(Sense::click()));
                        if response.clicked() {
                            self.update_entries(Some(entry.path.clone()), true);
                        }
                        Popup::context_menu(&response).show(|ui| {
                            file_context_menu(entry, ui, state, lsp_handles);
                        });
                    });
                    self.render(
                        entry.children.clone().unwrap(),
                        spacing + 1,
                        ui,
                        state,
                        lsp_handles,
                    );
                } else {
                    ui.horizontal(|ui| {
                        ui.label(" ".repeat(spacing));
                        ui.image(egui::include_image!("../../../../assets/Folder.svg"));
                        let response = ui.add(Label::new(&entry.name).sense(Sense::click()));
                        if response.clicked() {
                            self.update_entries(Some(entry.path.clone()), false);
                        }
                        Popup::context_menu(&response).show(|ui| {
                            file_context_menu(entry, ui, state, lsp_handles);
                        });
                    });
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label(" ".repeat(spacing));
                    ui.image(egui::include_image!("../../../../assets/FileText.svg"));
                    let response = ui.add(Label::new(&entry.name).sense(Sense::click()));
                    if response.clicked() {
                        perform_action(
                            Action::CreateBufferFromFile(entry.path.clone()),
                            state,
                            lsp_handles,
                        );
                    }
                    Popup::context_menu(&response).show(|ui| {
                        file_context_menu(entry, ui, state, lsp_handles);
                    });
                });
            }
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) {
        if state.workspace_folder != self.workspace_folder {
            self.workspace_folder = state.workspace_folder.clone();
            self.update_entries(None, true);
        }

        if state.preferences.show_file_explorer {
            egui::SidePanel::left("file_explorer")
                .resizable(false)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.render(self.entries.clone(), 0, ui, state, lsp_handles);
                    });
                });
        }
    }
}

impl Default for FileExplorer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn file_context_menu(
    entry: &FolderEntry,
    ui: &mut egui::Ui,
    state: &mut EditorState,
    _lsp_handles: &mut HashMap<Language, LSPClientHandle>,
) {
    ui.button("Create File").clicked();
    ui.button("Create Folder").clicked();
    ui.button("Rename").clicked();
    ui.button("Move").clicked();
    ui.button("Delete").clicked();
    if entry.is_dir && ui.button("Set as Workspace Folder").clicked() {
        state.workspace_folder = entry.path.clone();
    }
}
