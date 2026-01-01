use clap::Parser;
#[cfg(not(feature = "rift_rpc"))]
use reedline::{
    DefaultPrompt, DefaultPromptSegment, Reedline, Signal, ValidationResult, Validator,
};
#[cfg(not(feature = "rift_rpc"))]
use rsl::RSL;

#[derive(Parser)]
pub struct CLIArgs {
    pub script_path: Option<std::path::PathBuf>,
}

#[cfg(not(feature = "rift_rpc"))]
struct MultilineSourceValidator;

#[cfg(not(feature = "rift_rpc"))]
impl Validator for MultilineSourceValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if line.ends_with("\n\n") {
            ValidationResult::Complete
        } else {
            ValidationResult::Incomplete
        }
    }
}

#[cfg(not(feature = "rift_rpc"))]
fn main() {
    let cli_args = CLIArgs::parse();
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("Failed to create async runtime: {}", err);
            return;
        }
    };
    let mut rsl = RSL::new(None, rt.handle().clone());
    if let Some(path) = cli_args.script_path {
        match std::fs::read_to_string(&path) {
            Ok(source) => {
                if let Err(e) = rsl.run(source) {
                    eprintln!("{}", e);
                }
            }
            Err(err) => eprintln!("Failed to read script {}: {}", path.display(), err),
        }
    } else {
        let mut line_editor = Reedline::create().with_validator(Box::new(MultilineSourceValidator));
        let prompt = DefaultPrompt::new(DefaultPromptSegment::Empty, DefaultPromptSegment::Empty);

        loop {
            let signal = line_editor.read_line(&prompt);
            match signal {
                Ok(Signal::Success(source)) => {
                    if let Err(e) = rsl.run(source) {
                        eprintln!("{}", e);
                    }
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

#[cfg(feature = "rift_rpc")]
fn main() {}
