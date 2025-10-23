use crate::primitive::Primitive;

pub fn read_file(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        if let Primitive::String(path) = arguments.first().unwrap() {
            let content = std::fs::read_to_string(path).unwrap();
            return Primitive::String(content);
        }
        return Primitive::Error("Expected file path".to_string());
    }
    Primitive::Error("Expected 1 argument".to_string())
}

pub fn get_env_var(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        if let Primitive::String(key) = arguments.first().unwrap() {
            let value = std::env::var(key).unwrap();
            return Primitive::String(value);
        }
        return Primitive::Error("Expected key".to_string());
    }
    Primitive::Error("Expected 1 argument".to_string())
}
