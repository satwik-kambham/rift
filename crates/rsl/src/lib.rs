pub mod environment;
pub mod expression;
pub mod interpreter;
pub mod operator;
pub mod parser;
pub mod primitive;
pub mod scanner;
pub mod statement;
pub mod table;
pub mod token;

use std::rc::Rc;

use crate::environment::Environment;
use crate::primitive::Primitive;

pub struct RSL {
    pub environment: Rc<Environment>,
}

impl RSL {
    pub fn new() -> Self {
        let environment = Environment::new(None);
        environment.register_native_function("print", |arguments| {
            let text = arguments
                .iter()
                .map(|arg| format!("{}", arg))
                .collect::<Vec<_>>()
                .join(" ");
            println!("{}", text);
            return Primitive::Null;
        });
        Self {
            environment: Rc::new(environment),
        }
    }

    pub fn run(&self, source: String) {
        let mut scanner = crate::scanner::Scanner::new(source);
        let tokens = scanner.scan();

        let mut parser = crate::parser::Parser::new(tokens.clone());
        let statements = parser.parse();

        let mut interpreter =
            crate::interpreter::Interpreter::with_environment(statements, self.environment.clone());
        interpreter.interpret();
    }
}
