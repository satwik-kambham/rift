use std::cell::RefCell;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use which::which;

use crate::array::Array;
use crate::primitive::Primitive;
use crate::std_lib::args;
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
    let (workspace_dir, path) = args!(arguments; workspace_dir: String, path: String);

    if !is_absolute_path(&path) {
        return Primitive::String("Error: path is not absolute".to_string());
    }
    if !is_in_workspace(&workspace_dir, &path) {
        return Primitive::String("Error: path is not in workspace".to_string());
    }

    let output = match std::fs::read_to_string(&path) {
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
    Primitive::String(output)
}

pub fn agent_write_file(arguments: Vec<Primitive>) -> Primitive {
    let (workspace_dir, file_path, content) =
        args!(arguments; workspace_dir: String, file_path: String, content: String);

    if !is_absolute_path(&file_path) {
        return Primitive::String("Error: file_path is not absolute".to_string());
    }
    if !is_in_workspace(&workspace_dir, &file_path) {
        return Primitive::String("Error: file_path is not in workspace".to_string());
    }

    let parent_dir = Path::new(&file_path).parent().unwrap();
    if let Err(e) = std::fs::create_dir_all(parent_dir) {
        return Primitive::String(format!(
            "Error creating parent directories for '{}': {}",
            file_path, e
        ));
    }

    let output = match std::fs::write(&file_path, content) {
        Ok(_) => format!("Successfully wrote to file: {}", file_path),
        Err(e) => format!("Error writing to file '{}': {}", file_path, e),
    };
    Primitive::String(output)
}

pub fn agent_replace(arguments: Vec<Primitive>) -> Primitive {
    let (workspace_dir, file_path, old_string, new_string) = args!(
        arguments;
        workspace_dir: String,
        file_path: String,
        old_string: String,
        new_string: String
    );

    if !is_absolute_path(&file_path) {
        return Primitive::String("Error: file_path is not absolute.".to_string());
    }
    if !is_in_workspace(&workspace_dir, &file_path) {
        return Primitive::String("Error: file_path is not in workspace.".to_string());
    }

    let file_content = match std::fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            return Primitive::String(format!("Error reading file '{}': {}", file_path, e));
        }
    };

    let original_content = file_content.clone();
    let replaced_content = original_content.replace(&old_string, &new_string);

    let actual_replacements = original_content.matches(&old_string).count();

    if actual_replacements == 0 {
        return Primitive::String(format!(
            "Error: No occurrences of the old string found in '{}'.",
            file_path
        ));
    }

    let output = match std::fs::write(&file_path, replaced_content) {
        Ok(_) => format!(
            "Successfully replaced content in '{}'. {} occurrences replaced.",
            file_path, actual_replacements
        ),
        Err(e) => format!("Error writing to file '{}': {}", file_path, e),
    };
    Primitive::String(output)
}

pub fn create_blank_file(arguments: Vec<Primitive>) -> Primitive {
    let path = args!(arguments; path: String);

    let parent_dir = Path::new(&path).parent().unwrap_or_else(|| Path::new(""));
    if let Err(e) = std::fs::create_dir_all(parent_dir) {
        return Primitive::Error(format!("creating parent directories for '{}': {}", path, e));
    }

    match std::fs::File::create(&path) {
        Ok(_) => Primitive::Null,
        Err(e) => Primitive::Error(format!("creating file '{}': {}", path, e)),
    }
}

pub fn create_directory(arguments: Vec<Primitive>) -> Primitive {
    let path = args!(arguments; path: String);

    match std::fs::create_dir_all(&path) {
        Ok(_) => Primitive::Null,
        Err(e) => Primitive::Error(format!("creating directory '{}': {}", path, e)),
    }
}

pub fn read_file(arguments: Vec<Primitive>) -> Primitive {
    let path = args!(arguments; path: String);
    let content = std::fs::read_to_string(path).unwrap();
    Primitive::String(content)
}

pub fn get_env_var(arguments: Vec<Primitive>) -> Primitive {
    let key = args!(arguments; key: String);
    let value = std::env::var(key).unwrap();
    Primitive::String(value)
}

pub fn run_shell_command(arguments: Vec<Primitive>) -> Primitive {
    let (command, workspace_dir) = args!(arguments; command: String, workspace_dir: String);

    let (shell, flag) = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    match Command::new(shell)
        .arg(flag)
        .arg(&command)
        .current_dir(workspace_dir)
        .output()
    {
        Ok(output) => {
            let mut table = Table::new();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let status = output.status.code().unwrap_or(-1) as f32;
            table.set_value("stdout".to_string(), Primitive::String(stdout));
            table.set_value("stderr".to_string(), Primitive::String(stderr));
            table.set_value("status".to_string(), Primitive::Number(status));
            Primitive::Table(Rc::new(RefCell::new(table)))
        }
        Err(e) => Primitive::Error(format!("Error executing command: {}", e)),
    }
}

pub fn command_exists(arguments: Vec<Primitive>) -> Primitive {
    let command = args!(arguments; command: String);
    Primitive::Boolean(which(command).is_ok())
}

pub fn list_dir(arguments: Vec<Primitive>) -> Primitive {
    let path = args!(arguments; path: String);

    let mut result = Table::new();

    let entries = match std::fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(err) => {
            result.set_value(
                "entries".to_string(),
                Primitive::Array(Rc::new(RefCell::new(Array::new(vec![])))),
            );
            result.set_value(
                "error".to_string(),
                Primitive::String(format!("Error reading directory '{}': {}", path, err)),
            );
            return Primitive::Table(Rc::new(RefCell::new(result)));
        }
    };

    let mut items = vec![];

    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().to_string();
        let absolute_path = entry.path();

        items.push((name, absolute_path, file_type.is_dir()));
    }

    items.sort_by(
        |(left_name, _, left_is_dir), (right_name, _, right_is_dir)| match (
            left_is_dir,
            right_is_dir,
        ) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => left_name.to_lowercase().cmp(&right_name.to_lowercase()),
        },
    );

    let entries = items
        .into_iter()
        .map(|(name, path, is_dir)| {
            let mut table = Table::new();
            table.set_value("name".to_string(), Primitive::String(name));
            table.set_value(
                "path".to_string(),
                Primitive::String(path.to_string_lossy().to_string()),
            );
            table.set_value("is_dir".to_string(), Primitive::Boolean(is_dir));
            Primitive::Table(Rc::new(RefCell::new(table)))
        })
        .collect();

    result.set_value(
        "entries".to_string(),
        Primitive::Array(Rc::new(RefCell::new(Array::new(entries)))),
    );
    result.set_value("error".to_string(), Primitive::Null);

    Primitive::Table(Rc::new(RefCell::new(result)))
}

pub fn join_path(arguments: Vec<Primitive>) -> Primitive {
    let (base, segment) = args!(arguments; base: String, segment: String);

    let mut path = PathBuf::from(base);
    path.push(segment);

    Primitive::String(path.to_string_lossy().to_string())
}

pub fn parent_path(arguments: Vec<Primitive>) -> Primitive {
    let path = args!(arguments; path: String);

    let path = PathBuf::from(path);
    let parent = path.parent().map(PathBuf::from).unwrap_or(path);

    Primitive::String(parent.to_string_lossy().to_string())
}
