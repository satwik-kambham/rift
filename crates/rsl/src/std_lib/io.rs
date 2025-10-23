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
