use std::cell::RefCell;
use std::rc::Rc;

use crate::array::Array;
use crate::primitive::Primitive;
use crate::std_lib::args;

pub fn string_split_lines(arguments: Vec<Primitive>) -> Primitive {
    let string = args!(arguments; string: String);
    let lines: Vec<Primitive> = string
        .lines()
        .map(String::from)
        .map(Primitive::String)
        .collect();
    Primitive::Array(Rc::new(RefCell::new(Array::new(lines))))
}

pub fn string_len(arguments: Vec<Primitive>) -> Primitive {
    let string = args!(arguments; string: String);
    Primitive::Number(string.chars().count() as f32)
}
