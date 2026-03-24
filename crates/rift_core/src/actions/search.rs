use crate::state::EditorState;

use super::{Action, perform_action};

pub fn set_search_query(state: &mut EditorState, query: String) {
    state.search_query = query;
}

pub fn set_search_query_from_selection_or_prompt(state: &mut EditorState) -> Option<String> {
    let buffer_id = state.buffer_idx?;

    let selection_text = {
        let (buffer, instance) = state.get_buffer_by_id(buffer_id);
        let (start, end) = instance.selection.in_order();
        if start != end {
            Some(buffer.get_selection(&instance.selection))
        } else {
            None
        }
    };

    if let Some(selection_text) = selection_text {
        state.search_query = selection_text;
        perform_action(Action::FindNextWithQuery, state);
    } else {
        perform_action(
            Action::RunSource(
                "dialogModalOpen(\"Enter search query\", setSearchQueryFromDialog)".to_string(),
            ),
            state,
        );
    }
    None
}

pub fn find_next_with_query(state: &mut EditorState) -> Option<String> {
    let buffer_id = state.buffer_idx?;
    if state.search_query.is_empty() {
        return None;
    }

    let query = state.search_query.clone();
    let (buffer, instance) = state.get_buffer_by_id_mut(buffer_id);
    if let Some(selection) = buffer.find_next(&instance.cursor, &query) {
        instance.selection = selection;
        instance.cursor = selection.cursor;
    }
    None
}

pub fn search_workspace(state: &mut EditorState) {
    perform_action(
        Action::RunSource("createWorkspaceSearch()".to_string()),
        state,
    );
}
