pub mod array;
pub mod table;
pub mod web_requests;

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

pub fn to_json(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        let argument = arguments.first().unwrap();
        let serialized = serde_json::to_string(argument).unwrap();
        return Primitive::String(serialized);
    }
    Primitive::Error("Expected 1 argument".to_string())
}

pub fn from_json(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        if let Primitive::String(json) = arguments.first().unwrap() {
            let deserialized = serde_json::from_str(json).unwrap();
            return deserialized;
        }
        return Primitive::Error("Expected string".to_string());
    }
    Primitive::Error("Expected 1 argument".to_string())
}
