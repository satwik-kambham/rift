use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "rift_rpc")]
use tarpc::context;

use crate::RSL;
use crate::environment::Environment;
use crate::errors::RuntimeError;
use crate::operator::Operator;
use crate::primitive::Primitive;
use crate::statement::StatementResult;
#[cfg(feature = "rift_rpc")]
use crate::std_lib::args;
use crate::token::Span;

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
                match rsl.get_package_code(package_name) {
                    Ok(source) => {
                        let local_environment =
                            Rc::new(Environment::new(Some(environment.clone())));
                        rsl.run_with_environment(source, local_environment.clone())
                            .map_err(|e| {
                                RuntimeError::new(
                                    format!("Failed to import package {}: {}", package_name, e),
                                    self.span.clone(),
                                )
                            })?;
                        let exported_values = local_environment.get_exported_values();
                        let exported_values =
                            Primitive::Table(Rc::new(RefCell::new(exported_values)));
                        return Ok(exported_values);
                    }
                    Err(err) => {
                        eprintln!("Failed to import package {}: {}", package_name, err);
                        return Ok(Primitive::Null);
                    }
                }
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
                    &vec![],
                    environment,
                    rsl,
                    self.span.clone(),
                );
            }

            Ok(Primitive::Null)
        } else if let Primitive::Function(function_id) = environment.get_value(&self.identifier) {
            run_function_by_id(
                function_id,
                &self.parameters,
                environment,
                rsl,
                self.span.clone(),
            )
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

#[cfg(feature = "rift_rpc")]
fn execute_rpc_call(
    identifier: &str,
    parameters: Vec<Primitive>,
    rsl: &mut RSL,
    _span: Span,
) -> Result<Primitive, RuntimeError> {
    Ok(rsl.rt_handle.block_on(async {
        let ctx = context::Context::current();
        let client = &rsl.rift_rpc_client;

        match identifier {
            "log" => {
                let message = parameters
                    .iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<_>>()
                    .join(" ");
                client.rlog(ctx, message).await.unwrap();
                Primitive::Null
            }
            "openFile" => {
                let path = args!(parameters; path: String);
                client.open_file(ctx, path).await.unwrap();
                Primitive::Null
            }
            "setActiveBuffer" => {
                let buffer_id = args!(parameters; buffer_id: Number);
                client
                    .set_active_buffer(ctx, buffer_id as u32)
                    .await
                    .unwrap();
                Primitive::Null
            }
            "getActiveBuffer" => {
                let buffer_id = client.get_active_buffer(ctx).await.unwrap();
                if let Some(buffer_id) = buffer_id {
                    return Primitive::Number(buffer_id as f32);
                }
                Primitive::Null
            }
            "listBuffers" => {
                let buffers = client.list_buffers(ctx).await.unwrap();
                Primitive::String(buffers)
            }
            "getActions" => {
                let actions = client.get_actions(ctx).await.unwrap();
                Primitive::String(actions)
            }
            "getReferences" => {
                let references = client.get_references(ctx).await.unwrap();
                Primitive::String(references)
            }
            "getDefinitions" => {
                let definitions = client.get_definitions(ctx).await.unwrap();
                Primitive::String(definitions)
            }
            "getWorkspaceDiagnostics" => {
                let diagnostics = client.get_workspace_diagnostics(ctx).await.unwrap();
                Primitive::String(diagnostics)
            }
            "getViewportSize" => {
                let size = client.get_viewport_size(ctx).await.unwrap();
                Primitive::String(size)
            }
            "selectRange" => {
                let selection = args!(parameters; selection: String);
                client.select_range(ctx, selection).await.unwrap();
                Primitive::Null
            }
            "registerGlobalKeybind" => {
                let (definition, function_id) =
                    args!(parameters; definition: String, function_id: Function);
                client
                    .register_global_keybind(ctx, definition, function_id)
                    .await
                    .unwrap();
                Primitive::Null
            }
            "registerBufferKeybind" => {
                let (buffer_id, definition, function_id) = args!(
                    parameters;
                    buffer_id: Number,
                    definition: String,
                    function_id: Function
                );
                client
                    .register_buffer_keybind(ctx, buffer_id as u32, definition, function_id)
                    .await
                    .unwrap();
                Primitive::Null
            }
            "registerBufferInputHook" => {
                let (buffer_id, function_id) =
                    args!(parameters; buffer_id: Number, function_id: Function);
                client
                    .register_buffer_input_hook(ctx, buffer_id as u32, function_id)
                    .await
                    .unwrap();
                Primitive::Null
            }
            "createSpecialBuffer" => {
                let display_name = if parameters.is_empty() {
                    "".to_string()
                } else {
                    args!(parameters; display_name: String)
                };
                let buffer_id = client
                    .create_special_buffer(ctx, display_name)
                    .await
                    .unwrap();
                Primitive::Number(buffer_id as f32)
            }
            "setBufferContent" => {
                let (buffer_id, content) = args!(parameters; buffer_id: Number, content: String);
                client
                    .set_buffer_content(ctx, buffer_id as u32, content)
                    .await
                    .unwrap();
                Primitive::Null
            }
            "getBufferInput" => {
                let buffer_id = args!(parameters; buffer_id: Number);
                let input = client
                    .get_buffer_input(ctx, buffer_id as u32)
                    .await
                    .unwrap();
                Primitive::String(input)
            }
            "setBufferInput" => {
                let (buffer_id, input) = args!(parameters; buffer_id: Number, input: String);
                client
                    .set_buffer_input(ctx, buffer_id as u32, input)
                    .await
                    .unwrap();
                Primitive::Null
            }
            "setSearchQuery" => {
                let query = args!(parameters; query: String);
                client.set_search_query(ctx, query).await.unwrap();
                Primitive::Null
            }
            "getWorkspaceDir" => {
                let workspace_dir = client.get_workspace_dir(ctx).await.unwrap();
                Primitive::String(workspace_dir)
            }
            "runAction" => {
                let action = args!(parameters; action: String);
                let result = client.run_action(ctx, action).await.unwrap();
                Primitive::String(result)
            }
            "tts" => {
                let text = args!(parameters; text: String);
                client.tts(ctx, text).await.unwrap();
                Primitive::Null
            }
            _ => panic!("function {} does not exist", identifier),
        }
    }))
}

fn run_function_by_id(
    function_id: String,
    raw_parameters: &Vec<Box<dyn Expression>>,
    environment: Rc<Environment>,
    rsl: &mut RSL,
    span: Span,
) -> Result<Primitive, RuntimeError> {
    let local_environment = Rc::new(Environment::new(Some(environment.clone())));
    if let Some(function_definition) = local_environment.get_function(&function_id) {
        if raw_parameters.len() != function_definition.parameters.len() {
            return Err(RuntimeError::new(
                format!(
                    "Function '{}' expects {} parameters but received {}",
                    function_id,
                    function_definition.parameters.len(),
                    raw_parameters.len()
                ),
                span,
            ));
        }

        for i in 0..function_definition.parameters.len() {
            local_environment.set_value_local(
                function_definition.parameters.get(i).unwrap().clone(),
                raw_parameters
                    .get(i)
                    .unwrap()
                    .execute(environment.clone(), rsl)?,
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
        let mut parameters = vec![];
        for param_expression in raw_parameters {
            parameters.push(param_expression.execute(environment.clone(), rsl)?);
        }

        return Ok(native_function(parameters));
    }
    Err(RuntimeError::new(
        format!("Function '{}' not found in current scope", function_id),
        span,
    ))
}
