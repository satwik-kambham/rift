use std::rc::Rc;

use crate::statement::Statement;

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Null,
    Boolean(bool),
    Number(f32),
    String(String),
    Function(String),
}

#[derive(Clone)]
pub struct FunctionDefinition {
    pub parameters: Vec<String>,
    pub body: Rc<Vec<Box<dyn Statement>>>,
}
