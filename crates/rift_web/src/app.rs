use crate::command_dispatcher::{CommandDispatcher, KeybindInput};

pub struct App {
    dispatcher: CommandDispatcher,
    last_keybind: Option<KeybindInput>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            dispatcher: CommandDispatcher::default(),
            last_keybind: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rift Web");

            if let Some(keybind) = self.dispatcher.capture(ui) {
                self.last_keybind = Some(keybind);
            }

            if let Some(keybind) = &self.last_keybind {
                let mut modifiers: Vec<_> = keybind.modifiers.iter().cloned().collect();
                modifiers.sort();
                ui.label(format!(
                    "Last keybind: key=\"{}\" modifiers=[{}]",
                    keybind.key,
                    modifiers.join(",")
                ));
            }
        });
    }
}
