use std::collections::HashMap;
use std::fmt::Write;

use crate::primitive::Primitive;

#[derive(Debug)]
pub struct Table {
    table: HashMap<String, Primitive>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn set_value(&mut self, key: String, value: Primitive) {
        self.table.insert(key, value);
    }

    pub fn get_value(&self, key: &str) -> Primitive {
        self.table.get(key).unwrap_or(&Primitive::Null).clone()
    }

    pub fn keys(&self) -> Vec<String> {
        self.table.keys().cloned().collect()
    }

    pub fn merge(&mut self, other: &Table) {
        for (k, v) in other.table.iter() {
            self.table.entry(k.clone()).or_insert(v.clone());
        }
    }
}

impl PartialEq for Table {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output.push('{');

        for (i, (key, value)) in self.table.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            write!(&mut output, "{}: {}", key, value)?;
        }

        output.push('}');
        write!(f, "{}", output)
    }
}
