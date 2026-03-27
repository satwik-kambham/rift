use std::rc::Rc;

use crate::RSL;
use crate::environment::Environment;
use crate::errors::RuntimeError;
use crate::primitive::Primitive;
use crate::statement::StatementResult;
use crate::token::Span;

use super::Expression;

#[cfg(feature = "rift_rpc")]
use super::rpc::execute_rpc_call;

pub struct FunctionCallExpression {
    identifier: String,
    parameters: Vec<Box<dyn Expression>>,
    span: Span,
}

impl FunctionCallExpression {
    pub fn new(identifier: String, parameters: Vec<Box<dyn Expression>>, span: Span) -> Self {
        Self {
            identifier,
            parameters,
            span,
        }
    }

    fn collect_parameters(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<Vec<Primitive>, RuntimeError> {
        self.parameters
            .iter()
            .map(|param| param.execute(environment.clone(), rsl))
            .collect()
    }
}

impl Expression for FunctionCallExpression {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError> {
        if self.identifier == "import" {
            if self.parameters.len() != 1 {
                return Err(RuntimeError::new(
                    format!("Expected 1 parameter, got {}", self.parameters.len()),
                    self.span.clone(),
                ));
            }

            let parameters = self.collect_parameters(environment.clone(), rsl)?;

            if let Primitive::String(package_name) = parameters.first().unwrap() {
                return rsl.cached_import(package_name, self.span.clone());
            }

            Ok(Primitive::Null)
        } else if self.identifier == "runScript" {
            if self.parameters.len() != 1 {
                return Err(RuntimeError::new(
                    format!("Expected 1 parameter, got {}", self.parameters.len()),
                    self.span.clone(),
                ));
            }

            let parameters = self.collect_parameters(environment.clone(), rsl)?;

            if let Primitive::String(package_name) = parameters.first().unwrap() {
                return rsl.run_script(package_name, self.span.clone());
            }

            Ok(Primitive::Null)
        } else if self.identifier == "runFunctionById" {
            if self.parameters.len() != 1 {
                return Err(RuntimeError::new(
                    format!("Expected 1 parameter, got {}", self.parameters.len()),
                    self.span.clone(),
                ));
            }

            let parameters = self.collect_parameters(environment.clone(), rsl)?;

            if let Primitive::String(function_id) = parameters.first().unwrap() {
                return run_function_by_id(
                    function_id.clone(),
                    vec![],
                    environment,
                    rsl,
                    self.span.clone(),
                );
            }

            Ok(Primitive::Null)
        } else if let Primitive::Function(function_id) = environment.get_value(&self.identifier) {
            let parameters = self.collect_parameters(environment.clone(), rsl)?;
            run_function_by_id(function_id, parameters, environment, rsl, self.span.clone())
        } else {
            #[cfg(feature = "rift_rpc")]
            {
                execute_rpc_call(
                    &self.identifier,
                    self.collect_parameters(environment, rsl)?,
                    rsl,
                    self.span.clone(),
                )
            }
            #[cfg(not(feature = "rift_rpc"))]
            Err(RuntimeError::new(
                format!("Function '{}' not found in current scope", self.identifier),
                self.span.clone(),
            ))
        }
    }
}

pub struct TableMethodCallExpression {
    target: Box<dyn Expression>,
    key: String,
    parameters: Vec<Box<dyn Expression>>,
    span: Span,
}

impl TableMethodCallExpression {
    pub fn new(
        target: Box<dyn Expression>,
        key: String,
        parameters: Vec<Box<dyn Expression>>,
        span: Span,
    ) -> Self {
        Self {
            target,
            key,
            parameters,
            span,
        }
    }
}

impl Expression for TableMethodCallExpression {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError> {
        let target_value = self.target.execute(environment.clone(), rsl)?;
        let table = match target_value {
            Primitive::Table(table) => table,
            other => {
                return Err(RuntimeError::new(
                    format!("Expected table for method call, got {:?}", other),
                    self.span.clone(),
                ));
            }
        };

        let table_ref = table.borrow();
        if !table_ref.contains_key(&self.key) {
            return Err(RuntimeError::new(
                format!("Table has no key '{}'", self.key),
                self.span.clone(),
            ));
        }
        let value = table_ref.get_value(&self.key);
        drop(table_ref);

        let function_id = match value {
            Primitive::Function(function_id) => function_id,
            other => {
                return Err(RuntimeError::new(
                    format!(
                        "Expected function for table method '{}', got {:?}",
                        self.key, other
                    ),
                    self.span.clone(),
                ));
            }
        };

        let mut parameters = Vec::with_capacity(self.parameters.len() + 1);
        parameters.push(Primitive::Table(table.clone()));
        for param_expression in &self.parameters {
            parameters.push(param_expression.execute(environment.clone(), rsl)?);
        }

        run_function_by_id(function_id, parameters, environment, rsl, self.span.clone())
    }
}

pub struct IndexExpression {
    target: Box<dyn Expression>,
    index: Box<dyn Expression>,
    span: Span,
}

impl IndexExpression {
    pub fn new(target: Box<dyn Expression>, index: Box<dyn Expression>, span: Span) -> Self {
        Self {
            target,
            index,
            span,
        }
    }
}

impl Expression for IndexExpression {
    fn execute(
        &self,
        environment: Rc<Environment>,
        rsl: &mut RSL,
    ) -> Result<Primitive, RuntimeError> {
        let index_value = self.index.execute(environment.clone(), rsl)?;
        let target_value = self.target.execute(environment, rsl)?;

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

                let array_ref = array.borrow();
                if index >= array_ref.len() {
                    return Err(RuntimeError::new(
                        format!(
                            "Array index out of bounds: {index} (len = {})",
                            array_ref.len()
                        ),
                        self.span.clone(),
                    ));
                }
                Ok(array_ref.get(index))
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

                let table_ref = table.borrow();
                Ok(table_ref.get_value(&key))
            }
            other => Err(RuntimeError::new(
                format!("Expected array or table for indexing, got {:?}", other),
                self.span.clone(),
            )),
        }
    }
}

fn run_function_by_id(
    function_id: String,
    parameters: Vec<Primitive>,
    environment: Rc<Environment>,
    rsl: &mut RSL,
    span: Span,
) -> Result<Primitive, RuntimeError> {
    let local_environment = Rc::new(Environment::new(Some(environment.clone())));
    if let Some(function_definition) = local_environment.get_function(&function_id) {
        if parameters.len() != function_definition.parameters.len() {
            return Err(RuntimeError::new(
                format!(
                    "Function '{}' expects {} parameters but received {}",
                    function_id,
                    function_definition.parameters.len(),
                    parameters.len()
                ),
                span,
            ));
        }

        for i in 0..function_definition.parameters.len() {
            local_environment.set_value_local(
                function_definition.parameters.get(i).unwrap().clone(),
                parameters.get(i).unwrap().clone(),
                crate::environment::DeclarationType::Definition,
            );
        }

        for statement in function_definition.body.iter() {
            let result = statement.execute(local_environment.clone(), rsl)?;

            if let StatementResult::Return(result) = result {
                return Ok(result);
            }
        }
        return Ok(Primitive::Null);
    } else if let Some(native_function) = local_environment.get_native_function(&function_id) {
        return Ok(native_function(parameters));
    }
    Err(RuntimeError::new(
        format!("Function '{}' not found in current scope", function_id),
        span,
    ))
}
