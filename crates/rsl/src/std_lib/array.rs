use std::cell::RefCell;
use std::rc::Rc;

use crate::array::Array;
use crate::primitive::Primitive;
use crate::std_lib::args;

pub fn create_array(arguments: Vec<Primitive>) -> Primitive {
    Primitive::Array(Rc::new(RefCell::new(Array::new(arguments))))
}

pub fn array_len(arguments: Vec<Primitive>) -> Primitive {
    let array = args!(arguments; array: Array);
    Primitive::Number(array.borrow().len() as f32)
}

pub fn array_get(arguments: Vec<Primitive>) -> Primitive {
    let (array, index) = args!(arguments; array: Array, index: Number);
    array.borrow().get(index as usize)
}

pub fn array_set(arguments: Vec<Primitive>) -> Primitive {
    let (array, index, value) = args!(arguments; array: Array, index: Number, value);
    array.borrow_mut().set(index as usize, value);
    Primitive::Null
}

pub fn array_push_back(arguments: Vec<Primitive>) -> Primitive {
    let (array, value) = args!(arguments; array: Array, value);
    array.borrow_mut().push_back(value);
    Primitive::Null
}

pub fn array_remove(arguments: Vec<Primitive>) -> Primitive {
    let (array, index) = args!(arguments; array: Array, index: Number);
    array.borrow_mut().remove(index as usize)
}

pub fn array_pop_back(arguments: Vec<Primitive>) -> Primitive {
    let array = args!(arguments; array: Array);
    array.borrow_mut().pop_back()
}
