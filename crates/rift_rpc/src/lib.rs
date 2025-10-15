#[tarpc::service]
pub trait RiftRPC {
    async fn rlog(message: String);
    async fn set_active_buffer(id: u32);
    async fn register_global_keybind(definition: String, function_id: String);
    async fn create_special_buffer() -> u32;
}
