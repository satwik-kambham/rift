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
    println!("Source:\n{}", source);

    let mut scanner = crate::scanner::Scanner::new(source);
    let tokens = scanner.scan();

    println!("Tokens: {:?}", tokens);

    let mut parser = crate::parser::Parser::new(tokens.clone());
    let statements = parser.parse();
}
