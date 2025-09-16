use crate::operator::Operator;
use crate::primitive::Primitive;

pub trait Expression {}

pub struct LiteralExpression {
    literal: Primitive,
}

impl LiteralExpression {
    pub fn new(literal: Primitive) -> Self {
        Self { literal }
    }
}

impl Expression for LiteralExpression {}

pub struct VariableExpression {
    identifier: String,
}

impl VariableExpression {
    pub fn new(identifier: String) -> Self {
        Self { identifier }
    }
}

impl Expression for VariableExpression {}

pub struct BinaryExpression {
    left: Box<dyn Expression>,
    operator: Operator,
    right: Box<dyn Expression>,
}

impl BinaryExpression {
    pub fn new(left: Box<dyn Expression>, operator: Operator, right: Box<dyn Expression>) -> Self {
        Self {
            left,
            operator,
            right,
        }
    }
}

impl Expression for BinaryExpression {}

pub struct UnaryExpression {
    operator: Operator,
    expression: Box<dyn Expression>,
}

impl UnaryExpression {
    pub fn new(operator: Operator, expression: Box<dyn Expression>) -> Self {
        Self {
            operator,
            expression,
        }
    }
}

impl Expression for UnaryExpression {}

pub struct GroupingExpression {
    expression: Box<dyn Expression>,
}

impl GroupingExpression {
    pub fn new(expression: Box<dyn Expression>) -> Self {
        Self { expression }
    }
}

impl Expression for GroupingExpression {}

pub struct FunctionCallExpression {
    identifier: String,
    parameters: Vec<Box<dyn Expression>>,
}

impl FunctionCallExpression {
    pub fn new(identifier: String, parameters: Vec<Box<dyn Expression>>) -> Self {
        Self {
            identifier,
            parameters,
        }
    }
}

impl Expression for FunctionCallExpression {}
