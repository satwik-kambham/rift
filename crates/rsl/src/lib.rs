pub mod scanner;
pub mod token;

pub fn run_script(source: String) {
    println!("Source:\n{}", source);

    let mut scanner = scanner::Scanner::new(source);
    let tokens = scanner.scan();

    println!("Tokens: {:?}", tokens);
}
