use std::cell::RefCell;
use std::rc::Rc;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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

pub fn string_contains(arguments: Vec<Primitive>) -> Primitive {
    let (string, pattern) = args!(arguments; string: String, pattern: String);
    Primitive::Boolean(string.contains(&pattern))
}

pub fn string_to_lower(arguments: Vec<Primitive>) -> Primitive {
    let string = args!(arguments; string: String);
    Primitive::String(string.to_lowercase())
}

pub fn string_width(arguments: Vec<Primitive>) -> Primitive {
    let string = args!(arguments; string: String);
    Primitive::Number(UnicodeWidthStr::width(string.as_str()) as f32)
}

pub fn string_truncate_width(arguments: Vec<Primitive>) -> Primitive {
    let (string, max_width) = args!(arguments; string: String, max_width: Number);
    let max_width = if max_width < 0.0 {
        0
    } else {
        max_width as usize
    };

    let mut width = 0usize;
    let mut truncated = String::new();

    for ch in string.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_width {
            break;
        }
        width += ch_width;
        truncated.push(ch);
    }

    Primitive::String(truncated)
}
