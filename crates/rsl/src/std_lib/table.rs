use std::cell::RefCell;
use std::rc::Rc;

use crate::primitive::Primitive;
use crate::table::Table;

pub fn create_table(_arguments: Vec<Primitive>) -> Primitive {
    Primitive::Table(Rc::new(RefCell::new(Table::new())))
}

pub fn table_set(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 3 {
        if let Primitive::Table(table) = arguments.get(0).unwrap() {
            if let Primitive::String(key) = arguments.get(1).unwrap() {
                let value = arguments.get(2).unwrap().clone();
                table.borrow_mut().set_value(key.clone(), value);
                return Primitive::Null;
            }
            return Primitive::Error("Expected string key".to_string());
        }
        return Primitive::Error("Expected table".to_string());
    }
    Primitive::Error("Expected 3 arguments".to_string())
}

pub fn table_get(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 2 {
        if let Primitive::Table(table) = arguments.get(0).unwrap() {
            if let Primitive::String(key) = arguments.get(1).unwrap() {
                return table.borrow().get_value(key);
            }
            return Primitive::Error("Expected string key".to_string());
        }
        return Primitive::Error("Expected table".to_string());
    }
    Primitive::Error("Expected 2 arguments".to_string())
}

pub fn table_keys(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        if let Primitive::Table(table) = arguments.get(0).unwrap() {
            let keys = table.borrow().keys();
            let array = crate::array::Array::new(
                keys.into_iter()
                    .map(|k| Primitive::String(k))
                    .collect::<Vec<_>>(),
            );
            return Primitive::Array(Rc::new(RefCell::new(array)));
        }
        return Primitive::Error("Expected table".to_string());
    }
    Primitive::Error("Expected 1 argument".to_string())
}
