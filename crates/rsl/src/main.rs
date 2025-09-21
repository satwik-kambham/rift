use clap::Parser;
use reedline::{
    DefaultPrompt, DefaultPromptSegment, Reedline, Signal, ValidationResult, Validator,
};
use rsl::RSL;

#[derive(Parser)]
pub struct CLIArgs {
    pub script_path: Option<std::path::PathBuf>,
}

struct MultilineSourceValidator;

impl Validator for MultilineSourceValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if line.ends_with("\n\n") {
            ValidationResult::Complete
        } else {
            ValidationResult::Incomplete
        }
    }
}

fn main() {
    let cli_args = CLIArgs::parse();
    let rsl = RSL::new();
    if let Some(path) = cli_args.script_path {
        let source = std::fs::read_to_string(path).unwrap();
        rsl.run(source);
    } else {
        let mut line_editor = Reedline::create().with_validator(Box::new(MultilineSourceValidator));
        let prompt = DefaultPrompt::new(DefaultPromptSegment::Empty, DefaultPromptSegment::Empty);

        loop {
            let signal = line_editor.read_line(&prompt);
            match signal {
                Ok(Signal::Success(source)) => {
                    rsl.run(source);
                }
                Ok(Signal::CtrlC) => {
                    eprintln!("Aborted!");
                    break;
                }
                Ok(Signal::CtrlD) => {
                    println!("Goodbye!");
                    break;
                }
                Err(error) => {
                    eprintln!("Error: {:?}", error);
                }
            }
        }
    }
}
