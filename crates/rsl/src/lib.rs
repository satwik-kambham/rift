pub mod environment;
pub mod expression;
pub mod interpreter;
pub mod operator;
pub mod parser;
pub mod primitive;
pub mod scanner;
pub mod statement;
pub mod token;

use std::rc::Rc;

use crate::environment::Environment;

pub fn run_script(source: String) {
    let mut scanner = crate::scanner::Scanner::new(source);
    let tokens = scanner.scan();

    let mut parser = crate::parser::Parser::new(tokens.clone());
    let statements = parser.parse();

    let mut interpreter = crate::interpreter::Interpreter::new(statements);
    interpreter.interpret();
}

pub struct RSL {
    pub environment: Rc<Environment>,
}

impl RSL {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(Environment::new(None)),
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
