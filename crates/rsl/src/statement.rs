use std::rc::Rc;

use crate::RSL;
use crate::environment::{DeclarationType, Environment};
use crate::errors::RuntimeError;
use crate::expression;
use crate::primitive::{FunctionDefinition, Primitive};
use crate::token::Span;

#[derive(Debug)]
pub enum StatementResult {
    None,
    Break,
    Return(Primitive),
}

pub trait Statement {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<StatementResult, RuntimeError>;
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
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<StatementResult, RuntimeError> {
        self.expression.execute(environment, rsl)?;
        Ok(StatementResult::None)
    }
}

pub struct AssignmentStatement {
    identifier: String,
    expression: Box<dyn expression::Expression>,
    declaration_type: DeclarationType,
    span: Span,
}

impl AssignmentStatement {
    pub fn new(
        identifier: String,
        expression: Box<dyn expression::Expression>,
        declaration_type: DeclarationType,
        span: Span,
    ) -> Self {
        Self {
            identifier,
            expression,
            declaration_type,
            span,
        }
    }
}

impl Statement for AssignmentStatement {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<StatementResult, RuntimeError> {
        let local_environment = environment.clone();

        match self.declaration_type {
            DeclarationType::Assignment => {
                if !local_environment.has_value(&self.identifier) {
                    return Err(RuntimeError::new(
                        format!(
                            "Assignment to undefined variable '{}'; use let to define it first",
                            self.identifier
                        ),
                        self.span.clone(),
                    ));
                }
                local_environment.set_value_non_local(
                    self.identifier.clone(),
                    self.expression.execute(environment, rsl)?,
                    self.declaration_type,
                );
            }
            DeclarationType::Definition | DeclarationType::Export => {
                local_environment.set_value_local(
                    self.identifier.clone(),
                    self.expression.execute(environment, rsl)?,
                    self.declaration_type,
                );
            }
        }

        Ok(StatementResult::None)
    }
}

pub struct IndexAssignmentStatement {
    target: Box<dyn expression::Expression>,
    index: Box<dyn expression::Expression>,
    value: Box<dyn expression::Expression>,
    span: Span,
}

impl IndexAssignmentStatement {
    pub fn new(
        target: Box<dyn expression::Expression>,
        index: Box<dyn expression::Expression>,
        value: Box<dyn expression::Expression>,
        span: Span,
    ) -> Self {
        Self {
            target,
            index,
            value,
            span,
        }
    }
}

impl Statement for IndexAssignmentStatement {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<StatementResult, RuntimeError> {
        let index_value = self.index.execute(environment.clone(), rsl)?;
        let target_value = self.target.execute(environment.clone(), rsl)?;
        let value = self.value.execute(environment, rsl)?;

        match target_value {
            Primitive::Array(array) => {
                let index = match index_value {
                    Primitive::Number(value) => {
                        if value.is_sign_negative() || value.fract() != 0.0 {
                            return Err(RuntimeError::new(
                                format!("Array index must be a non-negative integer, got {value}"),
                                self.span.clone(),
                            ));
                        }
                        value as usize
                    }
                    other => {
                        return Err(RuntimeError::new(
                            format!("Array index must be a number, got {:?}", other),
                            self.span.clone(),
                        ));
                    }
                };

                let mut array_ref = array.borrow_mut();
                if index >= array_ref.len() {
                    return Err(RuntimeError::new(
                        format!(
                            "Array index out of bounds: {index} (len = {})",
                            array_ref.len()
                        ),
                        self.span.clone(),
                    ));
                }
                array_ref.set(index, value);
                Ok(StatementResult::None)
            }
            Primitive::Table(table) => {
                let key = match index_value {
                    Primitive::String(value) => value,
                    other => {
                        return Err(RuntimeError::new(
                            format!("Table index must be a string, got {:?}", other),
                            self.span.clone(),
                        ));
                    }
                };

                table.borrow_mut().set_value(key, value);
                Ok(StatementResult::None)
            }
            other => Err(RuntimeError::new(
                format!("Expected array or table for indexing, got {:?}", other),
                self.span.clone(),
            )),
        }
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
    fn execute(
        &self,
        environment: Rc<Environment>,
        _rsl: &mut RSL,
    ) -> Result<StatementResult, RuntimeError> {
        let local_environment = environment.clone();
        local_environment.register_function(
            self.identifier.clone(),
            FunctionDefinition {
                parameters: self.parameters.clone(),
                body: self.body.clone(),
            },
            self.export,
        );
        Ok(StatementResult::None)
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
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<StatementResult, RuntimeError> {
        Ok(StatementResult::Return(
            self.expression.execute(environment, rsl)?,
        ))
    }
}

pub struct IfStatement {
    condition: Box<dyn expression::Expression>,
    body: Vec<Box<dyn Statement>>,
    span: Span,
}

impl IfStatement {
    pub fn new(
        condition: Box<dyn expression::Expression>,
        body: Vec<Box<dyn Statement>>,
        span: Span,
    ) -> Self {
        Self {
            condition,
            body,
            span,
        }
    }
}

impl Statement for IfStatement {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<StatementResult, RuntimeError> {
        let condition = self.condition.execute(environment.clone(), rsl)?;
        if let Primitive::Boolean(condition) = condition {
            if condition {
                let local_environment = Rc::new(Environment::new(Some(environment.clone())));
                for statement in &self.body {
                    let statement_result = statement.execute(local_environment.clone(), rsl)?;
                    if matches!(
                        statement_result,
                        StatementResult::Break | StatementResult::Return(_)
                    ) {
                        return Ok(statement_result);
                    }
                }
            }
        } else {
            return Err(RuntimeError::new(
                format!(
                    "Expected boolean condition in if statement, got {:?}",
                    condition
                ),
                self.span.clone(),
            ));
        }

        Ok(StatementResult::None)
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
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<StatementResult, RuntimeError> {
        let local_environment = Rc::new(Environment::new(Some(environment.clone())));
        loop {
            for statement in &self.body {
                let execution_result = statement.execute(local_environment.clone(), rsl)?;

                if let StatementResult::Break = execution_result {
                    return Ok(StatementResult::None);
                }

                if matches!(execution_result, StatementResult::Return(_)) {
                    return Ok(execution_result);
                }
            }
        }
    }
}

pub struct BreakStatement {}

impl Default for BreakStatement {
    fn default() -> Self {
        Self::new()
    }
}

impl BreakStatement {
    pub fn new() -> Self {
        Self {}
    }
}

impl Statement for BreakStatement {
    fn execute(
        &self,
        _environment: Rc<Environment>,
        _rsl: &mut RSL,
    ) -> Result<StatementResult, RuntimeError> {
        Ok(StatementResult::Break)
    }
}
