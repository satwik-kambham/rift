use std::rc::Rc;

use crate::RSL;
use crate::environment::Environment;
use crate::errors::RuntimeError;
use crate::primitive::Primitive;

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
