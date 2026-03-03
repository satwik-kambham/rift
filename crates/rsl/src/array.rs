use std::fmt::Write;

use crate::primitive::Primitive;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Array {
    items: Vec<Primitive>,
}

impl Array {
    pub fn new(items: Vec<Primitive>) -> Self {
        Self { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get(&self, index: usize) -> Primitive {
        self.items
            .get(index)
            .unwrap_or(&Primitive::Error("Index out of bounds".to_string()))
            .clone()
    }

    pub fn set(&mut self, index: usize, value: Primitive) -> Primitive {
        if index < self.items.len() {
            self.items[index] = value;
            Primitive::Null
        } else {
            Primitive::Error("Index out of bounds".to_string())
        }
    }

    pub fn push_back(&mut self, value: Primitive) {
        self.items.push(value);
    }

    pub fn remove(&mut self, index: usize) -> Primitive {
        if index < self.items.len() {
            self.items.remove(index)
        } else {
            Primitive::Error("Index out of bounds".to_string())
        }
    }

    pub fn pop_back(&mut self) -> Primitive {
        if self.items.is_empty() {
            Primitive::Error("Array is empty".to_string())
        } else {
            self.items.pop().unwrap()
        }
    }

    pub fn insert(&mut self, index: usize, value: Primitive) -> Primitive {
        if index > self.items.len() {
            Primitive::Error("Index out of bounds".to_string())
        } else {
            self.items.insert(index, value);
            Primitive::Null
        }
    }
}

impl PartialEq for Array {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output.push('[');

        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            write!(&mut output, "{}", item)?;
        }

        output.push(']');
        write!(f, "{}", output)
    }
}
