use std::{fs::File, io::Read};

use egui::{FontData, FontDefinitions, FontTweak};
use rift_core::state::EditorState;

pub fn load_fonts(state: &mut EditorState) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    if let Ok(editor_font) = font_kit::source::SystemSource::new().select_best_match(
        &[font_kit::family_name::FamilyName::Title(
            state.preferences.editor_font_family.to_owned(),
        )],
        &font_kit::properties::Properties::new(),
    ) {
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, state.preferences.editor_font_family.to_owned());

        match editor_font {
            font_kit::handle::Handle::Path { path, font_index } => {
                let mut font_content = Vec::new();
                File::open(path)
                    .unwrap()
                    .read_to_end(&mut font_content)
                    .unwrap();
                fonts.font_data.insert(
                    state.preferences.editor_font_family.to_owned(),
                    std::sync::Arc::new(FontData {
                        font: std::borrow::Cow::Owned(font_content),
                        index: font_index,
                        tweak: FontTweak::default(),
                        weight: None,
                    }),
                );
            }
            font_kit::handle::Handle::Memory { bytes, font_index } => {
                fonts.font_data.insert(
                    state.preferences.editor_font_family.to_owned(),
                    std::sync::Arc::new(FontData {
                        font: std::borrow::Cow::Owned((*bytes).clone()),
                        index: font_index,
                        tweak: FontTweak::default(),
                        weight: None,
                    }),
                );
            }
        }
    }

    if let Ok(ui_font) = font_kit::source::SystemSource::new().select_best_match(
        &[font_kit::family_name::FamilyName::Title(
            state.preferences.ui_font_family.to_owned(),
        )],
        &font_kit::properties::Properties::new(),
    ) {
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, state.preferences.ui_font_family.to_owned());

        match ui_font {
            font_kit::handle::Handle::Path { path, font_index } => {
                let mut font_content = Vec::new();
                File::open(path)
                    .unwrap()
                    .read_to_end(&mut font_content)
                    .unwrap();
                fonts.font_data.insert(
                    state.preferences.ui_font_family.to_owned(),
                    std::sync::Arc::new(FontData {
                        font: std::borrow::Cow::Owned(font_content),
                        index: font_index,
                        tweak: FontTweak::default(),
                        weight: None,
                    }),
                );
            }
            font_kit::handle::Handle::Memory { bytes, font_index } => {
                fonts.font_data.insert(
                    state.preferences.ui_font_family.to_owned(),
                    std::sync::Arc::new(FontData {
                        font: std::borrow::Cow::Owned((*bytes).clone()),
                        index: font_index,
                        tweak: FontTweak::default(),
                        weight: None,
                    }),
                );
            }
        }
    }
    fonts
}
