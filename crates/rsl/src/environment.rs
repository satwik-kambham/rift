use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use uuid::Uuid;

use crate::primitive::{FunctionDefinition, Primitive};
use crate::table::Table;

#[derive(Clone, Copy)]
pub enum VariableType {
    Default,
    Export,
    Local,
}

pub struct Environment {
    values: RefCell<HashMap<String, (Primitive, VariableType)>>,
    functions: RefCell<HashMap<String, FunctionDefinition>>,
    native_functions: RefCell<HashMap<String, fn(Vec<Primitive>) -> Primitive>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new(parent: Option<Rc<Environment>>) -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
            functions: RefCell::new(HashMap::new()),
            native_functions: RefCell::new(HashMap::new()),
            parent,
        }
    }

    pub fn get_value(&self, name: &str) -> Primitive {
        if let Some(value) = self.values.borrow().get(name) {
            return value.0.clone();
        }

        if let Some(parent) = &self.parent {
            return parent.get_value(name);
        }

        Primitive::Null
    }

    pub fn get_exported_values(&self) -> Table {
        let mut exported_values = Table::new();

        for (name, (value, variable_type)) in self.values.borrow().iter() {
            if matches!(variable_type, VariableType::Export) {
                exported_values.set_value(name.clone(), value.clone());
            }
        }

        if let Some(parent) = &self.parent {
            let parent_exports = parent.get_exported_values();
            exported_values.merge(&parent_exports);
        }

        exported_values
    }

    pub fn set_value_local(&self, name: String, value: Primitive) {
        self.values
            .borrow_mut()
            .insert(name, (value, VariableType::Local));
    }

    pub fn set_value_non_local(&self, name: String, value: Primitive, variable_type: VariableType) {
        if self.values.borrow().contains_key(&name) {
            self.values
                .borrow_mut()
                .insert(name, (value, variable_type));
            return;
        }

        if let Some(parent) = &self.parent {
            parent.set_value_non_local(name, value, variable_type);
            return;
        }

        self.values
            .borrow_mut()
            .insert(name, (value, variable_type));
    }

    pub fn register_function(
        &self,
        name: String,
        function_definition: FunctionDefinition,
        export: bool,
    ) {
        if let Some(parent) = &self.parent {
            return parent.register_function(name, function_definition, export);
        }

        let function_id = Uuid::new_v4().to_string();

        self.functions
            .borrow_mut()
            .insert(function_id.clone(), function_definition);

        if export {
            self.set_value_non_local(name, Primitive::Function(function_id), VariableType::Export);
        } else {
            self.set_value_local(name, Primitive::Function(function_id));
        }
    }

    pub fn register_native_function(
        &self,
        name: &str,
        native_function: fn(Vec<Primitive>) -> Primitive,
    ) {
        if let Some(parent) = &self.parent {
            return parent.register_native_function(name, native_function);
        }

        let function_id = Uuid::new_v4().to_string();

        self.native_functions
            .borrow_mut()
            .insert(function_id.clone(), native_function);

        self.set_value_local(name.to_string(), Primitive::Function(function_id));
    }

    pub fn get_function(&self, function_id: &str) -> Option<FunctionDefinition> {
        if let Some(definition) = self.functions.borrow().get(function_id) {
            return Some(definition.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_function(function_id);
        }

        None
    }

    pub fn get_native_function(
        &self,
        function_id: &str,
    ) -> Option<fn(Vec<Primitive>) -> Primitive> {
        if let Some(func) = self.native_functions.borrow().get(function_id) {
            return Some(*func);
        }

        if let Some(parent) = &self.parent {
            return parent.get_native_function(function_id);
        }

        None
    }
}
