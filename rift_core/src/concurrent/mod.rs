pub mod web_api;

pub struct AsyncResult {
    pub result: String,
    pub callback: fn(String),
}
