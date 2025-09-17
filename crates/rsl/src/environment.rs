use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use uuid::Uuid;

use crate::primitive::{FunctionDefinition, Primitive};

pub struct Environment {
    values: RefCell<HashMap<String, Primitive>>,
    functions: RefCell<HashMap<String, FunctionDefinition>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new(parent: Option<Rc<Environment>>) -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
            functions: RefCell::new(HashMap::new()),
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
        if self.values.borrow().contains_key(&name) {
            self.values.borrow_mut().insert(name, value);
            return;
        }

        if let Some(parent) = &self.parent {
            parent.set_value_non_local(name, value);
            return;
        }

        self.values.borrow_mut().insert(name, value);
    }

    pub fn register_function(&self, name: String, function_definition: FunctionDefinition) {
        if let Some(parent) = &self.parent {
            return parent.register_function(name, function_definition);
        }

        let function_id = Uuid::new_v4().to_string();

        self.functions
            .borrow_mut()
            .insert(function_id.clone(), function_definition);

        self.set_value_local(name, Primitive::Function(function_id));
    }

    pub fn get_function(&self, function_id: &str) -> FunctionDefinition {
        if let Some(value) = self.functions.borrow().get(function_id) {
            return value.clone();
        }

        if let Some(parent) = &self.parent {
            return parent.get_function(function_id);
        }

        panic!("function id not found")
    }
}
