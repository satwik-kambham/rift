pub mod environment;
pub mod expression;
pub mod interpreter;
pub mod operator;
pub mod parser;
pub mod primitive;
pub mod scanner;
pub mod statement;
pub mod token;

pub fn run_script(source: String) {
    let mut scanner = crate::scanner::Scanner::new(source);
    let tokens = scanner.scan();

    let mut parser = crate::parser::Parser::new(tokens.clone());
    let statements = parser.parse();

    let mut interpreter = crate::interpreter::Interpreter::new(statements);
    interpreter.interpret();
}
