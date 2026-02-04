use std::cell::RefCell;
use std::rc::Rc;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::array::Array;
use crate::primitive::Primitive;
use crate::std_lib::args;
use rsl_macros::rsl_native;

#[rsl_native]
pub fn string_split_lines(arguments: Vec<Primitive>) -> Primitive {
    let string = args!(arguments; string: String);
    let lines: Vec<Primitive> = string
        .lines()
        .map(String::from)
        .map(Primitive::String)
        .collect();
    Primitive::Array(Rc::new(RefCell::new(Array::new(lines))))
}

#[rsl_native]
pub fn string_len(arguments: Vec<Primitive>) -> Primitive {
    let string = args!(arguments; string: String);
    Primitive::Number(string.chars().count() as f32)
}

#[rsl_native]
pub fn string_contains(arguments: Vec<Primitive>) -> Primitive {
    let (string, pattern) = args!(arguments; string: String, pattern: String);
    Primitive::Boolean(string.contains(&pattern))
}

#[rsl_native]
pub fn string_to_lower(arguments: Vec<Primitive>) -> Primitive {
    let string = args!(arguments; string: String);
    Primitive::String(string.to_lowercase())
}

#[rsl_native]
pub fn string_width(arguments: Vec<Primitive>) -> Primitive {
    let string = args!(arguments; string: String);
    Primitive::Number(UnicodeWidthStr::width(string.as_str()) as f32)
}

#[rsl_native]
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

fn wrap_lines(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();

    for raw_line in text.split('\n') {
        if raw_line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        let mut current_width = 0usize;

        for ch in raw_line.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);

            if current_width + ch_width > width && !current.is_empty() {
                lines.push(current);
                current = String::new();
                current_width = 0;
            }

            if ch_width > width && current.is_empty() {
                current.push(ch);
                lines.push(current);
                current = String::new();
                current_width = 0;
                continue;
            }

            current.push(ch);
            current_width += ch_width;
        }

        lines.push(current);
    }

    lines
}

#[rsl_native]
pub fn string_render_viewport(arguments: Vec<Primitive>) -> Primitive {
    let (string, viewport_width, viewport_height, scroll_amount) = args!(arguments; string: String, viewport_width: Number, viewport_height: Number, scroll_amount: Number);

    if viewport_width <= 0.0 || viewport_height <= 0.0 {
        return Primitive::String(String::new());
    }

    let width = viewport_width as usize;
    let height = viewport_height as usize;
    let scroll = scroll_amount as i32;

    let mut lines = wrap_lines(&string, width);

    if scroll >= 0 {
        let remove = scroll as usize;
        if remove >= lines.len() {
            lines.clear();
        } else {
            lines.drain(0..remove);
        }
    } else {
        let remove = (-scroll - 1).max(0) as usize;
        if remove >= lines.len() {
            lines.clear();
        } else {
            let new_len = lines.len() - remove;
            lines.truncate(new_len);
        }
    }

    let rendered = if scroll >= 0 {
        let end = height.min(lines.len());
        lines[..end].join("\n")
    } else {
        let start = lines.len().saturating_sub(height);
        lines[start..].join("\n")
    };

    Primitive::String(rendered)
}
