use std::rc::Rc;

use crate::RSL;
use crate::environment::{Environment, VariableType};
use crate::expression;
use crate::primitive::{FunctionDefinition, Primitive};

#[derive(Debug)]
pub enum StatementResult {
    None,
    Break,
    Return(Primitive),
}

pub trait Statement {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> StatementResult;
}

pub struct ExpressionStatement {
    expression: Box<dyn expression::Expression>,
}

impl ExpressionStatement {
    pub fn new(expression: Box<dyn expression::Expression>) -> Self {
        Self { expression }
    }
}

impl Statement for ExpressionStatement {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> StatementResult {
        self.expression.execute(environment, rsl);
        StatementResult::None
    }
}

pub struct AssignmentStatement {
    identifier: String,
    expression: Box<dyn expression::Expression>,
    variable_type: VariableType,
}

impl AssignmentStatement {
    pub fn new(
        identifier: String,
        expression: Box<dyn expression::Expression>,
        variable_type: VariableType,
    ) -> Self {
        Self {
            identifier,
            expression,
            variable_type,
        }
    }
}

impl Statement for AssignmentStatement {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> StatementResult {
        let local_environment = environment.clone();

        match self.variable_type {
            VariableType::Default | VariableType::Export => {
                local_environment.set_value_non_local(
                    self.identifier.clone(),
                    self.expression.execute(environment, rsl),
                    self.variable_type,
                );
            }
            VariableType::Local => {
                local_environment.set_value_local(
                    self.identifier.clone(),
                    self.expression.execute(environment, rsl),
                );
            }
        }

        StatementResult::None
    }
}

pub struct FunctionDefinitionStatement {
    identifier: String,
    parameters: Vec<String>,
    body: Rc<Vec<Box<dyn Statement>>>,
    export: bool,
}

impl FunctionDefinitionStatement {
    pub fn new(
        identifier: String,
        parameters: Vec<String>,
        body: Vec<Box<dyn Statement>>,
        export: bool,
    ) -> Self {
        Self {
            identifier,
            parameters,
            body: Rc::new(body),
            export,
        }
    }
}

impl Statement for FunctionDefinitionStatement {
    fn execute(&self, environment: Rc<Environment>, _rsl: &mut RSL) -> StatementResult {
        let local_environment = environment.clone();
        local_environment.register_function(
            self.identifier.clone(),
            FunctionDefinition {
                parameters: self.parameters.clone(),
                body: self.body.clone(),
            },
            self.export,
        );
        StatementResult::None
    }
}

pub struct ReturnStatement {
    expression: Box<dyn expression::Expression>,
}

impl ReturnStatement {
    pub fn new(expression: Box<dyn expression::Expression>) -> Self {
        Self { expression }
    }
}

impl Statement for ReturnStatement {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> StatementResult {
        StatementResult::Return(self.expression.execute(environment, rsl))
    }
}

pub struct IfStatement {
    condition: Box<dyn expression::Expression>,
    body: Vec<Box<dyn Statement>>,
}

impl IfStatement {
    pub fn new(condition: Box<dyn expression::Expression>, body: Vec<Box<dyn Statement>>) -> Self {
        Self { condition, body }
    }
}

impl Statement for IfStatement {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> StatementResult {
        let condition = self.condition.execute(environment.clone(), rsl);
        if let Primitive::Boolean(condition) = condition {
            if condition {
                let local_environment = Rc::new(Environment::new(Some(environment.clone())));
                for statement in &self.body {
                    let statement_result = statement.execute(local_environment.clone(), rsl);
                    if matches!(
                        statement_result,
                        StatementResult::Break | StatementResult::Return(_)
                    ) {
                        return statement_result;
                    }
                }
            }
        } else {
            panic!("Expected boolean got {:?}", condition)
        }

        StatementResult::None
    }
}

pub struct LoopStatement {
    body: Vec<Box<dyn Statement>>,
}

impl LoopStatement {
    pub fn new(body: Vec<Box<dyn Statement>>) -> Self {
        Self { body }
    }
}

impl Statement for LoopStatement {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> StatementResult {
        let local_environment = Rc::new(Environment::new(Some(environment.clone())));
        loop {
            for statement in &self.body {
                let execution_result = statement.execute(local_environment.clone(), rsl);

                if let StatementResult::Break = execution_result {
                    return StatementResult::None;
                }

                if matches!(execution_result, StatementResult::Return(_)) {
                    return execution_result;
                }
            }
        }
    }
}

pub struct BreakStatement {}

impl BreakStatement {
    pub fn new() -> Self {
        Self {}
    }
}

impl Statement for BreakStatement {
    fn execute(&self, _environment: Rc<Environment>, _rsl: &mut RSL) -> StatementResult {
        StatementResult::Break
    }
}
