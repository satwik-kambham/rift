use crate::expression;

pub trait Statement {}

pub struct ExpressionStatement {
    expression: Box<dyn expression::Expression>,
}

impl ExpressionStatement {
    pub fn new(expression: Box<dyn expression::Expression>) -> Self {
        Self { expression }
    }
}

impl Statement for ExpressionStatement {}

pub struct AssignmentStatement {
    identifier: String,
    expression: Box<dyn expression::Expression>,
}

impl AssignmentStatement {
    pub fn new(identifier: String, expression: Box<dyn expression::Expression>) -> Self {
        Self {
            identifier,
            expression,
        }
    }
}

impl Statement for AssignmentStatement {}

pub struct FunctionDefinition {
    identifier: String,
    parameters: Vec<String>,
    body: Vec<Box<dyn Statement>>,
}

impl FunctionDefinition {
    pub fn new(identifier: String, parameters: Vec<String>, body: Vec<Box<dyn Statement>>) -> Self {
        Self {
            identifier,
            parameters,
            body,
        }
    }
}

impl Statement for FunctionDefinition {}

pub struct ReturnStatement {
    expression: Box<dyn expression::Expression>,
}

impl ReturnStatement {
    pub fn new(expression: Box<dyn expression::Expression>) -> Self {
        Self { expression }
    }
}

impl Statement for ReturnStatement {}

pub struct IfStatement {
    condition: Box<dyn expression::Expression>,
    body: Vec<Box<dyn Statement>>,
}

impl IfStatement {
    pub fn new(condition: Box<dyn expression::Expression>, body: Vec<Box<dyn Statement>>) -> Self {
        Self { condition, body }
    }
}

impl Statement for IfStatement {}

pub struct LoopStatement {
    body: Vec<Box<dyn Statement>>,
}

impl LoopStatement {
    pub fn new(body: Vec<Box<dyn Statement>>) -> Self {
        Self { body }
    }
}

impl Statement for LoopStatement {}

pub struct BreakStatement {}

impl BreakStatement {
    pub fn new() -> Self {
        Self {}
    }
}

impl Statement for BreakStatement {}
