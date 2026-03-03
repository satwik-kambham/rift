use std::collections::HashSet;

/// Modifier strings mirror the current keybind handler expectations (`m`, `c`, `s`).
pub type ModifierSet = HashSet<String>;

#[derive(Debug, Clone, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub struct KeyInput {
    pub key: String,
    pub modifiers: ModifierSet,
}

#[derive(Debug, Clone, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub enum ClientToServer {
    KeyInput(KeyInput),
    Ping(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub enum ServerToClient {
    Ack(u64),
    Pong(u64),
}
