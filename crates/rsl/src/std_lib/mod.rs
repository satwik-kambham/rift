pub mod array;
pub mod table;

use crate::primitive::Primitive;

pub fn print(arguments: Vec<Primitive>) -> Primitive {
    let text = arguments
        .iter()
        .map(|arg| format!("{}", arg))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", text);
    Primitive::Null
}
