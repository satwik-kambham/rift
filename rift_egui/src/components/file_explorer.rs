use std::collections::HashMap;

use rift_core::{
    buffer::instance::Language, io::file_io::FolderEntry, lsp::client::LSPClientHandle,
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

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) -> f32 {
        let mut size = 0.0;
        
        if state.workspace_folder == self.workspace_folder {}

        egui::SidePanel::left("file_explorer")
            .resizable(true)
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                size = rect.right();
                ui.label("Hello");
            });
        size
    }
}
