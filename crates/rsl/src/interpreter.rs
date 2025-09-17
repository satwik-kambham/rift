use std::rc::Rc;

use crate::environment::Environment;
use crate::statement;

pub struct Interpreter {
    statements: Vec<Box<dyn statement::Statement>>,
    pub environment: Rc<Environment>,
}

impl Interpreter {
    pub fn new(statements: Vec<Box<dyn statement::Statement>>) -> Self {
        Self {
            statements,
            environment: Rc::new(Environment::new(None)),
        }
    }

    pub fn interpret(&mut self) {
        for statement in &self.statements {
            statement.execute(self.environment.clone());
        }
    }
}
