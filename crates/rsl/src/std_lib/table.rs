use std::cell::RefCell;
use std::rc::Rc;

use crate::primitive::Primitive;
use crate::std_lib::args;
use crate::table::Table;

pub fn create_table(_arguments: Vec<Primitive>) -> Primitive {
    Primitive::Table(Rc::new(RefCell::new(Table::new())))
}

pub fn table_set(arguments: Vec<Primitive>) -> Primitive {
    let (table, key, value) = args!(arguments; table: Table, key: String, value);
    table.borrow_mut().set_value(key, value);
    Primitive::Null
}

pub fn table_get(arguments: Vec<Primitive>) -> Primitive {
    let (table, key) = args!(arguments; table: Table, key: String);
    table.borrow().get_value(&key)
}

pub fn table_keys(arguments: Vec<Primitive>) -> Primitive {
    let table = args!(arguments; table: Table);
    let keys = table.borrow().keys();
    let array =
        crate::array::Array::new(keys.into_iter().map(Primitive::String).collect::<Vec<_>>());
    Primitive::Array(Rc::new(RefCell::new(array)))
}
