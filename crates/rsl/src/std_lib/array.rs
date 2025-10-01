use std::cell::RefCell;
use std::rc::Rc;

use crate::array::Array;
use crate::primitive::Primitive;

pub fn create_array(arguments: Vec<Primitive>) -> Primitive {
    Primitive::Array(Rc::new(RefCell::new(Array::new(arguments))))
}

pub fn array_len(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        if let Primitive::Array(array) = arguments.first().unwrap() {
            return Primitive::Number(array.borrow().len() as f32);
        }
        return Primitive::Error("Expected array".to_string());
    }
    Primitive::Error("Expected 1 argument".to_string())
}

pub fn array_get(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 2 {
        if let Primitive::Array(array) = arguments.first().unwrap() {
            if let Primitive::Number(index) = arguments.get(1).unwrap() {
                return array.borrow_mut().get(*index as usize);
            }
            return Primitive::Error("Expected number index".to_string());
        }
        return Primitive::Error("Expected array".to_string());
    }
    Primitive::Error("Expected 2 argument".to_string())
}

pub fn array_set(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 3 {
        if let Primitive::Array(array) = arguments.first().unwrap() {
            if let Primitive::Number(index) = arguments.get(1).unwrap() {
                let value = arguments.get(2).unwrap().clone();
                array.borrow_mut().set(*index as usize, value);
                return Primitive::Null;
            }
            return Primitive::Error("Expected number index".to_string());
        }
        return Primitive::Error("Expected array".to_string());
    }
    Primitive::Error("Expected 3 arguments".to_string())
}

pub fn array_push_back(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 2 {
        if let Primitive::Array(array) = arguments.first().unwrap() {
            let value = arguments.get(1).unwrap().clone();
            array.borrow_mut().push_back(value);
            return Primitive::Null;
        }
        return Primitive::Error("Expected array".to_string());
    }
    Primitive::Error("Expected 2 arguments".to_string())
}

pub fn array_remove(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 2 {
        if let Primitive::Array(array) = arguments.first().unwrap() {
            if let Primitive::Number(index) = arguments.get(1).unwrap() {
                return array.borrow_mut().remove(*index as usize);
            }
            return Primitive::Error("Expected number index".to_string());
        }
        return Primitive::Error("Expected array".to_string());
    }
    Primitive::Error("Expected 2 arguments".to_string())
}

pub fn array_pop_back(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        if let Primitive::Array(array) = arguments.first().unwrap() {
            return array.borrow_mut().pop_back();
        }
        return Primitive::Error("Expected array".to_string());
    }
    Primitive::Error("Expected 1 argument".to_string())
}
