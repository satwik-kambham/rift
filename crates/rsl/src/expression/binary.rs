use std::rc::Rc;

use crate::RSL;
use crate::environment::Environment;
use crate::errors::RuntimeError;
use crate::operator::Operator;
use crate::primitive::Primitive;
use crate::token::Span;

use super::Expression;

pub struct BinaryExpression {
    left: Box<dyn Expression>,
    operator: Operator,
    right: Box<dyn Expression>,
    span: Span,
}

impl BinaryExpression {
    pub fn new(
        left: Box<dyn Expression>,
        operator: Operator,
        right: Box<dyn Expression>,
        span: Span,
    ) -> Self {
        Self {
            left,
            operator,
            right,
            span,
        }
    }
}

impl Expression for BinaryExpression {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError> {
        let left_val = self.left.execute(environment.clone(), rsl)?;
        let right_val = self.right.execute(environment.clone(), rsl)?;

        match &self.operator {
            Operator::Or => {
                if let (Primitive::Boolean(left), Primitive::Boolean(right)) =
                    (&left_val, &right_val)
                {
                    Ok(Primitive::Boolean(*left || *right))
                } else {
                    Err(RuntimeError::new(
                        format!(
                            "Expected booleans for 'or', got left = {:?} and right = {:?}",
                            left_val, right_val
                        ),
                        self.span.clone(),
                    ))
                }
            }
            Operator::And => {
                if let (Primitive::Boolean(left), Primitive::Boolean(right)) =
                    (&left_val, &right_val)
                {
                    Ok(Primitive::Boolean(*left && *right))
                } else {
                    Err(RuntimeError::new(
                        format!(
                            "Expected booleans for 'and', got left = {:?} and right = {:?}",
                            left_val, right_val
                        ),
                        self.span.clone(),
                    ))
                }
            }
            Operator::IsEqual => Ok(Primitive::Boolean(left_val == right_val)),
            Operator::NotEqual => Ok(Primitive::Boolean(left_val != right_val)),
            Operator::LessThan => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (&left_val, &right_val)
                {
                    Ok(Primitive::Boolean(*left < *right))
                } else {
                    Err(RuntimeError::new(
                        format!(
                            "Expected numbers for '<', got left = {:?} and right = {:?}",
                            left_val, right_val
                        ),
                        self.span.clone(),
                    ))
                }
            }
            Operator::LessThanEqual => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (&left_val, &right_val)
                {
                    Ok(Primitive::Boolean(*left <= *right))
                } else {
                    Err(RuntimeError::new(
                        format!(
                            "Expected numbers for '<=', got left = {:?} and right = {:?}",
                            left_val, right_val
                        ),
                        self.span.clone(),
                    ))
                }
            }
            Operator::GreaterThan => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (&left_val, &right_val)
                {
                    Ok(Primitive::Boolean(*left > *right))
                } else {
                    Err(RuntimeError::new(
                        format!(
                            "Expected numbers for '>', got left = {:?} and right = {:?}",
                            left_val, right_val
                        ),
                        self.span.clone(),
                    ))
                }
            }
            Operator::GreaterThanEqual => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (&left_val, &right_val)
                {
                    Ok(Primitive::Boolean(*left >= *right))
                } else {
                    Err(RuntimeError::new(
                        format!(
                            "Expected numbers for '>=', got left = {:?} and right = {:?}",
                            left_val, right_val
                        ),
                        self.span.clone(),
                    ))
                }
            }
            Operator::Plus => match (&left_val, &right_val) {
                (Primitive::Number(left), Primitive::Number(right)) => {
                    Ok(Primitive::Number(*left + *right))
                }
                (Primitive::String(left), Primitive::String(right)) => {
                    Ok(Primitive::String(format!("{}{}", left, right)))
                }
                _ => Err(RuntimeError::new(
                    format!(
                        "Invalid operands for '+', got left = {} and right = {}",
                        left_val, right_val
                    ),
                    self.span.clone(),
                )),
            },
            Operator::Minus => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (&left_val, &right_val)
                {
                    Ok(Primitive::Number(*left - *right))
                } else {
                    Err(RuntimeError::new(
                        format!(
                            "Expected numbers for '-', got left = {:?} and right = {:?}",
                            left_val, right_val
                        ),
                        self.span.clone(),
                    ))
                }
            }
            Operator::Asterisk => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (&left_val, &right_val)
                {
                    Ok(Primitive::Number(*left * *right))
                } else {
                    Err(RuntimeError::new(
                        format!(
                            "Expected numbers for '*', got left = {:?} and right = {:?}",
                            left_val, right_val
                        ),
                        self.span.clone(),
                    ))
                }
            }
            Operator::Slash => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (&left_val, &right_val)
                {
                    Ok(Primitive::Number(*left / *right))
                } else {
                    Err(RuntimeError::new(
                        format!(
                            "Expected numbers for '/', got left = {:?} and right = {:?}",
                            left_val, right_val
                        ),
                        self.span.clone(),
                    ))
                }
            }
            Operator::Percent => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (&left_val, &right_val)
                {
                    Ok(Primitive::Number(*left % *right))
                } else {
                    Err(RuntimeError::new(
                        format!(
                            "Expected numbers for '%', got left = {:?} and right = {:?}",
                            left_val, right_val
                        ),
                        self.span.clone(),
                    ))
                }
            }
            other => Err(RuntimeError::new(
                format!("Unexpected operator {:?}", other),
                self.span.clone(),
            )),
        }
    }
}

pub struct UnaryExpression {
    operator: Operator,
    expression: Box<dyn Expression>,
    span: Span,
}

impl UnaryExpression {
    pub fn new(operator: Operator, expression: Box<dyn Expression>, span: Span) -> Self {
        Self {
            operator,
            expression,
            span,
        }
    }
}

impl Expression for UnaryExpression {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError> {
        let expression = self.expression.execute(environment.clone(), rsl)?;

        match &self.operator {
            Operator::Minus => {
                if let Primitive::Number(expression) = expression {
                    Ok(Primitive::Number(-expression))
                } else {
                    Err(RuntimeError::new(
                        format!("Expected number for '-', got {:?}", expression),
                        self.span.clone(),
                    ))
                }
            }
            Operator::Not => {
                if let Primitive::Boolean(expression) = expression {
                    Ok(Primitive::Boolean(!expression))
                } else {
                    Err(RuntimeError::new(
                        format!("Expected boolean for 'not', got {:?}", expression),
                        self.span.clone(),
                    ))
                }
            }
            other => Err(RuntimeError::new(
                format!("Unexpected operator {:?}", other),
                self.span.clone(),
            )),
        }
    }
}
