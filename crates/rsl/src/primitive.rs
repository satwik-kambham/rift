#[derive(Clone)]
pub enum Primitive {
    Null,
    Boolean(bool),
    Number(f32),
    String(String),
}
