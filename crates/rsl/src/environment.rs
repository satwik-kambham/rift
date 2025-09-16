use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::primitive::Primitive;

pub struct Environment {
    values: RefCell<HashMap<String, Primitive>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new(parent: Option<Rc<Environment>>) -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
            parent,
        }
    }

    pub fn get_value(&self, name: &str) -> Primitive {
        if let Some(value) = self.values.borrow().get(name) {
            return value.clone();
        }

        if let Some(parent) = &self.parent {
            return parent.get_value(name);
        }

        Primitive::Null
    }

    pub fn set_value_local(&self, name: String, value: Primitive) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn set_value_non_local(&self, name: String, value: Primitive) {
        todo!("Recusively check parents and assign to global if not defined");
    }
}
