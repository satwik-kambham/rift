use std::cell::RefCell;
use std::rc::Rc;

use crate::RSL;
use crate::array::Array;
use crate::environment::Environment;
use crate::errors::RuntimeError;
use crate::primitive::Primitive;
use crate::table::Table;
use crate::token::Span;

mod binary;
mod call;
#[cfg(feature = "rift_rpc")]
mod rpc;

pub use binary::*;
pub use call::*;

pub trait Expression {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError>;
}

pub struct LiteralExpression {
    literal: Primitive,
}

impl LiteralExpression {
    pub fn new(literal: Primitive) -> Self {
        Self { literal }
    }
}

impl Expression for LiteralExpression {
    fn execute(
        &self,
        _environment: Rc<Environment>,
        _rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError> {
        Ok(self.literal.clone())
    }
}

pub struct VariableExpression {
    identifier: String,
}

impl VariableExpression {
    pub fn new(identifier: String) -> Self {
        Self { identifier }
    }
}

impl Expression for VariableExpression {
    fn execute(
        &self,
        environment: Rc<Environment>,
        _rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError> {
        Ok(environment.get_value(&self.identifier))
    }
}

pub struct GroupingExpression {
    expression: Box<dyn Expression>,
}

impl GroupingExpression {
    pub fn new(expression: Box<dyn Expression>) -> Self {
        Self { expression }
    }
}

impl Expression for GroupingExpression {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError> {
        self.expression.execute(environment, rsl)
    }
}

pub struct ArrayLiteralExpression {
    elements: Vec<Box<dyn Expression>>,
}

impl ArrayLiteralExpression {
    pub fn new(elements: Vec<Box<dyn Expression>>, _span: Span) -> Self {
        Self { elements }
    }
}

impl Expression for ArrayLiteralExpression {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError> {
        let mut items = Vec::with_capacity(self.elements.len());
        for element in &self.elements {
            items.push(element.execute(Rc::clone(&environment), rsl)?);
        }
        Ok(Primitive::Array(Rc::new(RefCell::new(Array::new(items)))))
    }
}

pub struct TableLiteralExpression {
    entries: Vec<(Box<dyn Expression>, Box<dyn Expression>)>,
    span: Span,
}

impl TableLiteralExpression {
    pub fn new(entries: Vec<(Box<dyn Expression>, Box<dyn Expression>)>, span: Span) -> Self {
        Self { entries, span }
    }
}

impl Expression for TableLiteralExpression {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError> {
        let mut table = Table::new();
        for (key_expr, value_expr) in &self.entries {
            let key = key_expr.execute(Rc::clone(&environment), rsl)?;
            let value = value_expr.execute(Rc::clone(&environment), rsl)?;
            match key {
                Primitive::String(s) => table.set_value(s, value),
                _ => {
                    return Err(RuntimeError::new(
                        "table key must be a string".to_string(),
                        self.span.clone(),
                    ));
                }
            }
        }
        Ok(Primitive::Table(Rc::new(RefCell::new(table))))
    }
}
