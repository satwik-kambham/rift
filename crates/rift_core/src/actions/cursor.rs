use crate::state::EditorState;

pub fn move_down(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_down(&mut instance.cursor, instance.column_level);
    instance.sync_cursor_vertical(instance.cursor);
    None
}

pub fn move_up(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_up(&mut instance.cursor, instance.column_level);
    instance.sync_cursor_vertical(instance.cursor);
    None
}

pub fn move_left(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_left(&mut instance.cursor);
    instance.sync_cursor(instance.cursor);
    None
}

pub fn move_right(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_right(&mut instance.cursor);
    instance.sync_cursor(instance.cursor);
    None
}

pub fn extend_down(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_down(&mut instance.cursor, instance.column_level);
    instance.extend_selection_vertical(instance.cursor);
    None
}

pub fn extend_up(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_up(&mut instance.cursor, instance.column_level);
    instance.extend_selection_vertical(instance.cursor);
    None
}

pub fn extend_left(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_left(&mut instance.cursor);
    instance.extend_selection(instance.cursor);
    None
}

pub fn extend_right(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_right(&mut instance.cursor);
    instance.extend_selection(instance.cursor);
    None
}

pub fn move_line_start(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_line_start(&mut instance.cursor);
    instance.sync_cursor(instance.cursor);
    None
}

pub fn move_line_end(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_line_end(&mut instance.cursor);
    instance.sync_cursor(instance.cursor);
    None
}

pub fn extend_line_start(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_line_start(&mut instance.cursor);
    instance.extend_selection(instance.cursor);
    None
}

pub fn extend_line_end(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_line_end(&mut instance.cursor);
    instance.extend_selection(instance.cursor);
    None
}

pub fn go_to_buffer_start(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_buffer_start(&mut instance.cursor);
    instance.sync_cursor(instance.cursor);
    None
}

pub fn go_to_buffer_end(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.move_cursor_buffer_end(&mut instance.cursor);
    instance.sync_cursor(instance.cursor);
    None
}

pub fn unselect(state: &mut EditorState) -> Option<String> {
    let (_buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    instance.sync_cursor(instance.cursor);
    None
}

pub fn scroll_up(state: &mut EditorState) -> Option<String> {
    let (_buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    instance.scroll.row = instance.scroll.row.saturating_sub(1);
    None
}

pub fn scroll_down(state: &mut EditorState) -> Option<String> {
    let (_buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    instance.scroll.row = instance.scroll.row.saturating_add(1);
    None
}
