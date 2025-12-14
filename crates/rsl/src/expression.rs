use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "rift_rpc")]
use tarpc::context;

use crate::RSL;
use crate::environment::Environment;
use crate::operator::Operator;
use crate::primitive::Primitive;
use crate::statement::StatementResult;
#[cfg(feature = "rift_rpc")]
use crate::std_lib::args;

pub trait Expression {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> Primitive;
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
    fn execute(&self, _environment: Rc<Environment>, _rsl: &mut RSL) -> Primitive {
        self.literal.clone()
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
    fn execute(&self, environment: Rc<Environment>, _rsl: &mut RSL) -> Primitive {
        environment.get_value(&self.identifier)
    }
}

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

impl Expression for BinaryExpression {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> Primitive {
        let left = self.left.execute(environment.clone(), rsl);
        let right = self.right.execute(environment.clone(), rsl);

        match &self.operator {
            Operator::Or => {
                if let (Primitive::Boolean(left), Primitive::Boolean(right)) = (left, right) {
                    Primitive::Boolean(left || right)
                } else {
                    panic!("Expected left and right expression of 'or' operator to be boolean")
                }
            }
            Operator::And => {
                if let (Primitive::Boolean(left), Primitive::Boolean(right)) = (left, right) {
                    Primitive::Boolean(left && right)
                } else {
                    panic!("Expected left and right expression of 'and' operator to be boolean")
                }
            }
            Operator::IsEqual => Primitive::Boolean(left == right),
            Operator::NotEqual => Primitive::Boolean(left != right),
            Operator::LessThan => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (left, right) {
                    Primitive::Boolean(left < right)
                } else {
                    panic!("Expected left and right expression of '<' operator to be numbers")
                }
            }
            Operator::LessThanEqual => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (left, right) {
                    Primitive::Boolean(left <= right)
                } else {
                    panic!("Expected left and right expression of '<=' operator to be numbers")
                }
            }
            Operator::GreaterThan => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (left, right) {
                    Primitive::Boolean(left > right)
                } else {
                    panic!("Expected left and right expression of '>' operator to be numbers")
                }
            }
            Operator::GreaterThanEqual => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (left, right) {
                    Primitive::Boolean(left >= right)
                } else {
                    panic!("Expected left and right expression of '>=' operator to be numbers")
                }
            }
            Operator::Plus => match (&left, &right) {
                (Primitive::Number(left), Primitive::Number(right)) => {
                    Primitive::Number(left + right)
                }
                (Primitive::String(left), Primitive::String(right)) => {
                    Primitive::String(format!("{}{}", left, right))
                }
                _ => {
                    panic!(
                        "Invalid operands for '+', got left = {} and right = {}",
                        left, right
                    )
                }
            },
            Operator::Minus => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (left, right) {
                    Primitive::Number(left - right)
                } else {
                    panic!("Expected left and right expression of '-' operator to be numbers")
                }
            }
            Operator::Asterisk => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (left, right) {
                    Primitive::Number(left * right)
                } else {
                    panic!("Expected left and right expression of '*' operator to be numbers")
                }
            }
            Operator::Slash => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (left, right) {
                    Primitive::Number(left / right)
                } else {
                    panic!("Expected left and right expression of '/' operator to be numbers")
                }
            }
            Operator::Percent => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (left, right) {
                    Primitive::Number(left % right)
                } else {
                    panic!("Expected left and right expression of '%' operator to be numbers")
                }
            }
            other => panic!("Unexpected operator {:?}", other),
        }
    }
}

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

impl Expression for UnaryExpression {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> Primitive {
        let expression = self.expression.execute(environment.clone(), rsl);

        match &self.operator {
            Operator::Minus => {
                if let Primitive::Number(expression) = expression {
                    Primitive::Number(-expression)
                } else {
                    panic!("Expected expression of '-' operator to be a number")
                }
            }
            Operator::Not => {
                if let Primitive::Boolean(expression) = expression {
                    Primitive::Boolean(!expression)
                } else {
                    panic!("Expected expression of 'not' operator to be boolean")
                }
            }
            other => panic!("Unexpected operator {:?}", other),
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
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> Primitive {
        self.expression.execute(environment, rsl)
    }
}

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

    fn collect_parameters(&self, environment: Rc<Environment>, rsl: &mut RSL) -> Vec<Primitive> {
        self.parameters
            .iter()
            .map(|param| param.execute(environment.clone(), rsl))
            .collect()
    }
}

impl Expression for FunctionCallExpression {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> Primitive {
        if self.identifier == "import" {
            if self.parameters.len() != 1 {
                panic!("Expected 1 parameter, got {}", self.parameters.len())
            }

            let parameters = self.collect_parameters(environment.clone(), rsl);

            if let Primitive::String(package_name) = parameters.first().unwrap() {
                match rsl.get_package_code(package_name) {
                    Ok(source) => {
                        let local_environment =
                            Rc::new(Environment::new(Some(environment.clone())));
                        rsl.run_with_environment(source, local_environment.clone());
                        let exported_values = local_environment.get_exported_values();
                        let exported_values =
                            Primitive::Table(Rc::new(RefCell::new(exported_values)));
                        return exported_values;
                    }
                    Err(err) => {
                        eprintln!("Failed to import package {}: {}", package_name, err);
                        return Primitive::Null;
                    }
                }
            }

            Primitive::Null
        } else if self.identifier == "runFunctionById" {
            if self.parameters.len() != 1 {
                panic!("Expected 1 parameter, got {}", self.parameters.len())
            }

            let parameters = self.collect_parameters(environment.clone(), rsl);

            if let Primitive::String(function_id) = parameters.first().unwrap() {
                return run_function_by_id(function_id.clone(), &vec![], environment, rsl);
            }

            Primitive::Null
        } else if let Primitive::Function(function_id) = environment.get_value(&self.identifier) {
            run_function_by_id(function_id, &self.parameters, environment, rsl)
        } else {
            #[cfg(feature = "rift_rpc")]
            {
                execute_rpc_call(
                    &self.identifier,
                    self.collect_parameters(environment, rsl),
                    rsl,
                )
            }
            #[cfg(not(feature = "rift_rpc"))]
            panic!("function {} does not exist", self.identifier)
        }
    }
}

#[cfg(feature = "rift_rpc")]
fn execute_rpc_call(identifier: &str, parameters: Vec<Primitive>, rsl: &mut RSL) -> Primitive {
    rsl.rt_handle.block_on(async {
        let ctx = context::Context::current();
        let client = &rsl.rift_rpc_client;

        match identifier {
            "log" => {
                let message = args!(parameters; message: String);
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
            "getWorkspaceDir" => {
                let workspace_dir = client.get_workspace_dir(ctx).await.unwrap();
                Primitive::String(workspace_dir)
            }
            "runAction" => {
                let action = args!(parameters; action: String);
                let result = client.run_action(ctx, action).await.unwrap();
                Primitive::String(result)
            }
            _ => panic!("function {} does not exist", identifier),
        }
    })
}

fn run_function_by_id(
    function_id: String,
    raw_parameters: &Vec<Box<dyn Expression>>,
    environment: Rc<Environment>,
    rsl: &mut RSL,
) -> Primitive {
    let local_environment = Rc::new(Environment::new(Some(environment.clone())));
    if let Some(function_definition) = local_environment.get_function(&function_id) {
        if raw_parameters.len() != function_definition.parameters.len() {
            panic!("Number of parameters does not match")
        }

        for i in 0..function_definition.parameters.len() {
            local_environment.set_value_local(
                function_definition.parameters.get(i).unwrap().clone(),
                raw_parameters
                    .get(i)
                    .unwrap()
                    .execute(environment.clone(), rsl),
            );
        }

        for statement in function_definition.body.iter() {
            let result = statement.execute(local_environment.clone(), rsl);

            if let StatementResult::Return(result) = result {
                return result;
            }
        }
        return Primitive::Null;
    } else if let Some(native_function) = local_environment.get_native_function(&function_id) {
        let mut parameters = vec![];
        for param_expression in raw_parameters {
            parameters.push(param_expression.execute(environment.clone(), rsl));
        }

        return native_function(parameters);
    }
    panic!("function {} not found", function_id);
}
