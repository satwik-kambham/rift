use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use uuid::Uuid;

use crate::primitive::{FunctionDefinition, Primitive};
use crate::table::Table;

pub type NativeFunction = fn(Vec<Primitive>) -> Primitive;
type NativeFunctionMap = HashMap<String, NativeFunction>;

#[derive(Clone, Copy)]
pub enum DeclarationType {
    Definition,
    Assignment,
    Export,
}

pub struct Environment {
    values: RefCell<HashMap<String, (Primitive, DeclarationType)>>,
    functions: RefCell<HashMap<String, FunctionDefinition>>,
    native_functions: RefCell<NativeFunctionMap>,
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

        for (name, (value, declaration_type)) in self.values.borrow().iter() {
            if matches!(declaration_type, DeclarationType::Export) {
                exported_values.set_value(name.clone(), value.clone());
            }
        }

        exported_values
    }

    pub fn has_value_local(&self, name: &str) -> bool {
        self.values.borrow().contains_key(name)
    }

    pub fn has_value(&self, name: &str) -> bool {
        if self.has_value_local(name) {
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.has_value(name);
        }

        false
    }

    pub fn set_value_local(
        &self,
        name: String,
        value: Primitive,
        declaration_type: DeclarationType,
    ) {
        self.values
            .borrow_mut()
            .insert(name, (value, declaration_type));
    }

    pub fn set_value_non_local(
        &self,
        name: String,
        value: Primitive,
        declaration_type: DeclarationType,
    ) {
        if self.values.borrow().contains_key(&name) {
            self.values
                .borrow_mut()
                .insert(name, (value, declaration_type));
            return;
        }

        if let Some(parent) = &self.parent {
            parent.set_value_non_local(name, value, declaration_type);
            return;
        }

        self.values
            .borrow_mut()
            .insert(name, (value, declaration_type));
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
            self.set_value_local(
                name,
                Primitive::Function(function_id),
                DeclarationType::Export,
            );
        } else {
            self.set_value_local(
                name,
                Primitive::Function(function_id),
                DeclarationType::Definition,
            );
        }
    }

    pub fn register_native_function(&self, name: &str, native_function: NativeFunction) {
        if let Some(parent) = &self.parent {
            return parent.register_native_function(name, native_function);
        }

        let function_id = Uuid::new_v4().to_string();

        self.native_functions
            .borrow_mut()
            .insert(function_id.clone(), native_function);

        self.set_value_local(
            name.to_string(),
            Primitive::Function(function_id),
            DeclarationType::Definition,
        );
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

    pub fn get_native_function(&self, function_id: &str) -> Option<NativeFunction> {
        if let Some(func) = self.native_functions.borrow().get(function_id) {
            return Some(*func);
        }

        if let Some(parent) = &self.parent {
            return parent.get_native_function(function_id);
        }

        None
    }
}
