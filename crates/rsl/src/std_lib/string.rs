use std::cell::RefCell;
use std::rc::Rc;

use crate::array::Array;
use crate::primitive::Primitive;

pub fn string_split_lines(arguments: Vec<Primitive>) -> Primitive {
    if let Primitive::String(string) = arguments.first().unwrap() {
        let lines: Vec<Primitive> = string
            .lines()
            .map(String::from)
            .map(Primitive::String)
            .collect();
        let lines = Primitive::Array(Rc::new(RefCell::new(Array::new(lines))));
        return lines;
    }
    return Primitive::Error("Expected string".to_string());
}
