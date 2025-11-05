use std::cell::RefCell;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

use crate::primitive::Primitive;
use crate::table::Table;

pub fn is_absolute_path(path: &str) -> bool {
    Path::new(path).is_absolute()
}

pub fn is_in_workspace(workspace_dir: &str, path: &str) -> bool {
    let workspace_path = Path::new(workspace_dir);
    let file_path = Path::new(path);
    file_path.starts_with(workspace_path)
}

pub fn agent_read_file(arguments: Vec<Primitive>) -> Primitive {
    if let Primitive::String(workspace_dir) = arguments.first().unwrap() {
        if let Primitive::String(path) = arguments.get(1).unwrap() {
            if !is_absolute_path(path) {
                return Primitive::String("Error: path is not absolute".to_string());
            }
            if !is_in_workspace(workspace_dir, path) {
                return Primitive::String("Error: path is not in workspace".to_string());
            }

            let output = match std::fs::read_to_string(path) {
                Ok(buf) => {
                    let lines: Vec<String> = buf
                        .lines()
                        .enumerate()
                        .map(|(line_number, line)| format!("{}\t{}", line_number + 1, line))
                        .collect();
                    format!("{}\n\n{}", path, lines.join("\n"))
                }
                Err(e) => format!("Error reading file '{}': {}", path, e),
            };
            return Primitive::String(output);
        }
    }
    Primitive::String("Uncorrect arguments".to_string())
}

pub fn agent_write_file(arguments: Vec<Primitive>) -> Primitive {
    if let Primitive::String(workspace_dir) = arguments.first().unwrap() {
        if let Primitive::String(file_path) = arguments.get(1).unwrap() {
            if let Primitive::String(content) = arguments.get(2).unwrap() {
                if !is_absolute_path(file_path) {
                    return Primitive::String("Error: file_path is not absolute".to_string());
                }
                if !is_in_workspace(workspace_dir, file_path) {
                    return Primitive::String("Error: file_path is not in workspace".to_string());
                }

                let parent_dir = Path::new(file_path).parent().unwrap();
                if let Err(e) = std::fs::create_dir_all(parent_dir) {
                    return Primitive::String(format!(
                        "Error creating parent directories for '{}': {}",
                        file_path, e
                    ));
                }

                let output = match std::fs::write(file_path, content) {
                    Ok(_) => format!("Successfully wrote to file: {}", file_path),
                    Err(e) => format!("Error writing to file '{}': {}", file_path, e),
                };
                return Primitive::String(output);
            }
        }
    }
    Primitive::String("Uncorrect arguments".to_string())
}

pub fn agent_replace(arguments: Vec<Primitive>) -> Primitive {
    if let Primitive::String(workspace_dir) = arguments.first().unwrap() {
        if let Primitive::String(file_path) = arguments.get(1).unwrap() {
            if let Primitive::String(old_string) = arguments.get(2).unwrap() {
                if let Primitive::String(new_string) = arguments.get(3).unwrap() {
                    if !is_absolute_path(file_path) {
                        return Primitive::String("Error: file_path is not absolute.".to_string());
                    }
                    if !is_in_workspace(workspace_dir, file_path) {
                        return Primitive::String(
                            "Error: file_path is not in workspace.".to_string(),
                        );
                    }

                    let file_content = match std::fs::read_to_string(file_path) {
                        Ok(content) => content,
                        Err(e) => {
                            return Primitive::String(format!(
                                "Error reading file '{}': {}",
                                file_path, e
                            ));
                        }
                    };

                    let original_content = file_content.clone();
                    let replaced_content = original_content.replace(old_string, new_string);

                    let actual_replacements = original_content.matches(old_string).count();

                    if actual_replacements == 0 {
                        return Primitive::String(format!(
                            "Error: No occurrences of the old string found in '{}'.",
                            file_path
                        ));
                    }

                    let file_content = replaced_content;

                    let output = match std::fs::write(file_path, file_content) {
                        Ok(_) => format!(
                            "Successfully replaced content in '{}'. {} occurrences replaced.",
                            file_path, actual_replacements
                        ),
                        Err(e) => format!("Error writing to file '{}': {}", file_path, e),
                    };
                    return Primitive::String(output);
                }
            }
        }
    }
    Primitive::String("Uncorrect arguments".to_string())
}

pub fn read_file(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        if let Primitive::String(path) = arguments.first().unwrap() {
            let content = std::fs::read_to_string(path).unwrap();
            return Primitive::String(content);
        }
        return Primitive::Error("Expected file path".to_string());
    }
    Primitive::Error("Expected 1 argument".to_string())
}

pub fn get_env_var(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        if let Primitive::String(key) = arguments.first().unwrap() {
            let value = std::env::var(key).unwrap();
            return Primitive::String(value);
        }
        return Primitive::Error("Expected key".to_string());
    }
    Primitive::Error("Expected 1 argument".to_string())
}

pub fn run_shell_command(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        if let Primitive::String(command) = arguments.first().unwrap() {
            if let Primitive::String(workspace_dir) = arguments.get(1).unwrap() {
                match Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .current_dir(workspace_dir)
                    .output()
                {
                    Ok(output) => {
                        let mut table = Table::new();
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        table.set_value("stdout".to_string(), Primitive::String(stdout));
                        table.set_value("stderr".to_string(), Primitive::String(stderr));
                        return Primitive::Table(Rc::new(RefCell::new(table)));
                    }
                    Err(e) => {
                        return Primitive::Error(format!("Error executing command: {}", e));
                    }
                }
            }
        }
        return Primitive::Error("Expected file path".to_string());
    }
    Primitive::Error("Expected 1 argument".to_string())
}
