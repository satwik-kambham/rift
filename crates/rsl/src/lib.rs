pub mod array;
pub mod environment;
pub mod expression;
pub mod interpreter;
pub mod operator;
pub mod parser;
pub mod primitive;
pub mod scanner;
pub mod statement;
pub mod std_lib;
pub mod table;
pub mod token;

use std::rc::Rc;

use crate::environment::Environment;

pub struct RSL {
    pub environment: Rc<Environment>,
}

impl RSL {
    pub fn new() -> Self {
        let environment = Environment::new(None);

        environment.register_native_function("print", std_lib::print);
        environment.register_native_function("createArray", std_lib::array::create_array);
        environment.register_native_function("arrayLen", std_lib::array::array_len);
        environment.register_native_function("arrayGet", std_lib::array::array_get);
        environment.register_native_function("arraySet", std_lib::array::array_set);
        environment.register_native_function("arrayPushBack", std_lib::array::array_push_back);
        environment.register_native_function("arrayRemove", std_lib::array::array_remove);
        environment.register_native_function("arrayPopBack", std_lib::array::array_pop_back);

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
