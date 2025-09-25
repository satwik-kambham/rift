#[tarpc::service]
pub trait RiftRPC {
    async fn rlog(message: String);
}
