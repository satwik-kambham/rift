use crate::actions::Action;
use crate::state::Mode;

use std::collections::HashSet;
use std::str::FromStr;

/// Dynamic keybind definition with keybind chaining
///
/// Format:
/// ACTION_ID MODE Modifier-Key Key ...
///
/// Case agnostic
///
/// MODE - ALL, NOR, INS
///
/// Modifier - C (Control), M (Alt), S (Shift)
pub struct Keybind {
    pub action: Action,
    pub mode: HashSet<Mode>,
    pub sequence: String,
    pub definition: String,
}

impl Keybind {
    pub fn from_definition(definition: &str) -> Self {
        let parsed_definition = definition.to_lowercase();
        let (action_id, parsed_definition) = parsed_definition.split_once(" ").unwrap();
        let (mode, sequence) = parsed_definition.split_once(" ").unwrap();

        let action = Action::from_str(action_id).expect(action_id);

        let mode = match mode {
            "all" => HashSet::from([Mode::Normal, Mode::Insert]),
            "nor" => HashSet::from([Mode::Normal]),
            "ins" => HashSet::from([Mode::Insert]),
            _ => HashSet::from([Mode::Normal, Mode::Insert]),
        };

        Self {
            action,
            mode,
            sequence: sequence.to_string(),
            definition: definition.to_string(),
        }
    }
}

pub struct KeybindHandler {
    pub running_sequence: String,
    pub keybinds: Vec<Keybind>,
}

impl KeybindHandler {
    pub fn new(keybinds: Vec<&str>) -> Self {
        Self {
            running_sequence: "".to_string(),
            keybinds: keybinds
                .iter()
                .map(|definition| Keybind::from_definition(definition))
                .collect(),
        }
    }

    pub fn handle_input(
        &mut self,
        mode: Mode,
        key: String,
        modifiers: HashSet<String>,
    ) -> Option<Action> {
        let mut input = key.clone().to_lowercase();
        if !modifiers.is_empty() {
            if !(modifiers.contains("s")
                && "<>?:\"{}|~!@#$%^&*()_+".chars().any(|c| key.contains(c)))
            {
                input.insert(0, '-');
            }
            if modifiers.contains("c") {
                input.insert(0, 'c');
            }
            if modifiers.contains("m") {
                input.insert(0, 'm');
            }
            if modifiers.contains("s") && !"<>?:\"{}|~!@#$%^&*()_+".chars().any(|c| key.contains(c))
            {
                input.insert(0, 's');
            }
        }
        if !self.running_sequence.is_empty() {
            self.running_sequence.push(' ')
        };
        self.running_sequence.push_str(&input);

        // If running sequence matches a keybind, perform the corresponding action
        // If running sequence is part of a keybind, update the running sequence dont perform any action
        // If running sequence is not part of any keybind, clear the running sequence
        // If running sequence is not part of any keybind and mode is insert mode, clear the running sequence and
        // insert the character if applicable
        if let Some(keybind) = self.keybinds.iter().find(|keybind| {
            keybind.mode.contains(&mode) && keybind.sequence == self.running_sequence
        }) {
            self.running_sequence = "".to_string();
            Some(keybind.action.clone())
        } else if self.keybinds.iter().any(|keybind| {
            keybind.mode.contains(&mode)
                && keybind
                    .sequence
                    .starts_with(&(self.running_sequence.clone() + " "))
        }) {
            return None;
        } else if matches!(mode, Mode::Insert) && key.is_ascii() && key.len() == 1 {
            self.running_sequence = "".to_string();
            if key.chars().all(|c| c.is_ascii_alphabetic()) {
                return Some(Action::InsertTextAtCursorAndTriggerCompletion(key));
            } else {
                return Some(Action::InsertTextAtCursor(key));
            }
        } else {
            self.running_sequence = "".to_string();
            return None;
        }
    }
}

impl Default for KeybindHandler {
    fn default() -> Self {
        Self::new(vec![
            "quit nor space q",
            "enter-insert-mode nor i",
            "quit-insert-mode all escape",
            "insert-new-line-at-cursor ins enter",
            "increase-font-size nor +",
            "decrease-font-size nor -",
            "scroll-up nor c-home",
            "scroll-down nor c-end",
            "move-cursor-down all down",
            "extend-cursor-down all s-down",
            "move-cursor-up all up",
            "extend-cursor-up all s-up",
            "move-cursor-left all left",
            "extend-cursor-left all s-left",
            "move-cursor-right all right",
            "extend-cursor-right all s-right",
            "move-cursor-line-start all home",
            "extend-cursor-line-start all s-home",
            "move-cursor-line-end all end",
            "extend-cursor-line-end all s-end",
            "delete-previous-character ins backspace",
            "delete-next-character ins delete",
            "insert-space ins space",
            "add-tab ins tab",
            "open-file nor space s-f",
            "fuzzy-find-file nor space f",
            "switch-buffer nor space b",
            "move-cursor-down nor j",
            "extend-cursor-down nor s-j",
            "move-cursor-up nor k",
            "extend-cursor-up nor s-k",
            "move-cursor-left nor h",
            "extend-cursor-left nor s-h",
            "move-cursor-right nor l",
            "extend-cursor-right nor s-l",
            "add-new-line-below-and-enter-insert-mode nor o",
            "delete-selection nor d",
            "delete-selection-and-enter-insert-mode nor c",
            "workspace-diagnostics nor space d",
            "select-current-line nor x",
            "select-and-extend-current-line nor s-x",
            "select-till-end-of-word nor w",
            "extend-select-till-end-of-word nor s-w",
            "select-till-start-of-word nor b",
            "extend-select-till-start-of-word nor s-b",
            "insert-after-selection nor a",
            "go-to-buffer-start nor g g",
            "go-to-buffer-end nor g e",
            "format-current-buffer nor s",
            "save-current-buffer nor s-s",
            "undo nor u",
            "redo nor s-u",
            "add-indent nor >",
            "remove-indent nor <",
            "cycle-previous-buffer nor ,",
            "cycle-next-buffer nor .",
            "search-workspace nor /",
            "unselect nor ;",
            "open-command-dispatcher nor :",
            "lsp-hover nor z",
            "lsp-completion nor s-z",
            "go-to-definition nor g d",
            "go-to-references nor g r",
            "copy-to-register nor y",
            "copy-to-clipboard nor s-y",
            "paste-from-register nor p",
            "paste-from-clipboard nor s-p",
            "keybind-help nor ?",
        ])
    }
}
