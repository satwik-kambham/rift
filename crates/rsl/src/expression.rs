use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "rift_rpc")]
use tarpc::context;

use crate::RSL;
use crate::environment::Environment;
use crate::operator::Operator;
use crate::primitive::Primitive;
use crate::statement::StatementResult;

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
            Operator::Plus => {
                if let (Primitive::Number(left), Primitive::Number(right)) = (left, right) {
                    Primitive::Number(left + right)
                } else {
                    panic!("Expected left and right expression of '+' operator to be numbers")
                }
            }
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
}

impl Expression for FunctionCallExpression {
    fn execute(&self, environment: Rc<Environment>, rsl: &mut RSL) -> Primitive {
        if self.identifier == "import" {
            if self.parameters.len() != 1 {
                panic!("Expected 1 parameter, got {}", self.parameters.len())
            }

            let mut parameters = vec![];
            for param_expression in &self.parameters {
                parameters.push(param_expression.execute(environment.clone(), rsl));
            }

            if let Primitive::String(package_name) = parameters.first().unwrap() {
                let source = rsl.get_package_code(package_name);
                let local_environment = Rc::new(Environment::new(Some(environment.clone())));
                rsl.run_with_environment(source, local_environment.clone());
                let exported_values = local_environment.get_exported_values();
                let exported_values = Primitive::Table(Rc::new(RefCell::new(exported_values)));
                return exported_values;
            }

            Primitive::Null
        } else if self.identifier == "runFunctionById" {
            if self.parameters.len() != 1 {
                panic!("Expected 1 parameter, got {}", self.parameters.len())
            }

            let mut parameters = vec![];
            for param_expression in &self.parameters {
                parameters.push(param_expression.execute(environment.clone(), rsl));
            }

            if let Primitive::String(function_id) = parameters.first().unwrap() {
                return run_function_by_id(function_id.clone(), &vec![], environment, rsl);
            }

            Primitive::Null
        } else if let Primitive::Function(function_id) = environment.get_value(&self.identifier) {
            return run_function_by_id(function_id, &self.parameters, environment, rsl);
        } else {
            #[cfg(feature = "rift_rpc")]
            {
                let mut parameters = vec![];
                for param_expression in &self.parameters {
                    parameters.push(param_expression.execute(environment.clone(), rsl));
                }

                return rsl.rt_handle.block_on(async {
                    match self.identifier.as_str() {
                        "log" => {
                            rsl.rift_rpc_client
                                .rlog(
                                    context::Context::current(),
                                    parameters.first().unwrap().to_string(),
                                )
                                .await
                                .unwrap();
                            Primitive::Null
                        }
                        "setActiveBuffer" => {
                            if let Primitive::Number(number) = parameters.first().unwrap() {
                                rsl.rift_rpc_client
                                    .set_active_buffer(context::Context::current(), *number as u32)
                                    .await
                                    .unwrap();
                            }
                            Primitive::Null
                        }
                        "registerGlobalKeybind" => {
                            if let Primitive::String(definition) = parameters.first().unwrap() {
                                if let Primitive::Function(function_id) = parameters.get(1).unwrap()
                                {
                                    rsl.rift_rpc_client
                                        .register_global_keybind(
                                            context::Context::current(),
                                            definition.clone(),
                                            function_id.clone(),
                                        )
                                        .await
                                        .unwrap();
                                }
                            }
                            Primitive::Null
                        }
                        _ => panic!("function {} does not exist", self.identifier),
                    }
                });
            }
            #[cfg(not(feature = "rift_rpc"))]
            panic!("function {} does not exist", self.identifier)
        }
    }
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
