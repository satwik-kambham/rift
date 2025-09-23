use std::rc::Rc;

use crate::statement::Statement;
use crate::table::Table;

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Null,
    Boolean(bool),
    Number(f32),
    String(String),
    Function(String),
    Table(Rc<Table>),
}

impl std::fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Null => write!(f, "null"),
            Primitive::Boolean(b) => write!(f, "{b}"),
            Primitive::Number(n) => write!(f, "{n}"),
            Primitive::String(s) => write!(f, "{s}"),
            Primitive::Function(name) => write!(f, "<fn {name}>"),
            Primitive::Table(table) => write!(f, "{table}"),
        }
    }
}

#[derive(Clone)]
pub struct FunctionDefinition {
    pub parameters: Vec<String>,
    pub body: Rc<Vec<Box<dyn Statement>>>,
}
