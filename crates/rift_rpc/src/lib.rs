#[tarpc::service]
pub trait RiftRPC {
    async fn rlog(message: String);
    async fn set_active_buffer(id: u32);
    async fn register_global_keybind(definition: String, function_id: String);
    async fn create_special_buffer(display_name: String) -> u32;
    async fn register_buffer_keybind(buffer_id: u32, definition: String, function_id: String);
    async fn set_buffer_content(buffer_id: u32, content: String);
    async fn get_buffer_input(buffer_id: u32) -> String;
    async fn set_buffer_input(buffer_id: u32, input: String);
    async fn register_buffer_input_hook(buffer_id: u32, function_id: String);
    async fn get_workspace_dir() -> String;
    async fn run_action(action: String) -> String;
    async fn tts(text: String);
    async fn get_active_buffer() -> Option<u32>;
    async fn list_buffers() -> String;
    async fn get_actions() -> String;
    async fn get_definitions() -> String;
    async fn get_references() -> String;
    async fn get_workspace_diagnostics() -> String;
    async fn get_viewport_size() -> String;
    async fn select_range(selection: String);
    async fn open_file(path: String);
    async fn set_search_query(query: String);
}
