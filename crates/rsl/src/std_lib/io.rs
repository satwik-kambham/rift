use std::cell::RefCell;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use serde_json::{Value, json};
use which::which;

use crate::array::Array;
use crate::primitive::Primitive;
use crate::std_lib::args;
use crate::table::Table;
use rsl_macros::rsl_native;

pub fn is_absolute_path(path: &str) -> bool {
    Path::new(path).is_absolute()
}

pub fn is_in_workspace(workspace_dir: &str, path: &str) -> bool {
    let workspace_path = Path::new(workspace_dir);
    let file_path = Path::new(path);
    file_path.starts_with(workspace_path)
}

fn tool_response(
    ok: bool,
    tool: &str,
    error_code: Option<&str>,
    message: String,
    hints: Vec<&str>,
    data: Value,
) -> Primitive {
    let output = json!({
        "ok": ok,
        "tool": tool,
        "error_code": error_code,
        "message": message,
        "hints": hints,
        "data": data
    });
    Primitive::String(output.to_string())
}

#[rsl_native]
pub fn agent_read_file_tool(arguments: Vec<Primitive>) -> Primitive {
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

#[rsl_native]
pub fn agent_write_file_tool(arguments: Vec<Primitive>) -> Primitive {
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

#[rsl_native]
pub fn agent_replace_tool(arguments: Vec<Primitive>) -> Primitive {
    let mut iter = arguments.into_iter();
    let workspace_dir = match iter.next() {
        Some(Primitive::String(value)) => value,
        Some(_) => {
            return tool_response(
                false,
                "replace",
                Some("INVALID_ARGUMENT"),
                "workspace_dir must be a string.".to_string(),
                vec!["Pass an absolute workspace directory path as a string."],
                json!({}),
            );
        }
        None => {
            return tool_response(
                false,
                "replace",
                Some("INVALID_ARGUMENT"),
                "Missing workspace_dir.".to_string(),
                vec!["Provide workspace_dir, path, old_string, and new_string."],
                json!({}),
            );
        }
    };
    let file_path = match iter.next() {
        Some(Primitive::String(value)) => value,
        Some(_) => {
            return tool_response(
                false,
                "replace",
                Some("INVALID_ARGUMENT"),
                "path must be a string.".to_string(),
                vec!["Use an absolute path from find_file or search_workspace."],
                json!({}),
            );
        }
        None => {
            return tool_response(
                false,
                "replace",
                Some("INVALID_ARGUMENT"),
                "Missing path.".to_string(),
                vec!["Provide path, old_string, and new_string."],
                json!({}),
            );
        }
    };
    let old_string = match iter.next() {
        Some(Primitive::String(value)) => value,
        Some(_) => {
            return tool_response(
                false,
                "replace",
                Some("INVALID_ARGUMENT"),
                "old_string must be a string.".to_string(),
                vec!["Pass the exact old_string to replace."],
                json!({}),
            );
        }
        None => {
            return tool_response(
                false,
                "replace",
                Some("INVALID_ARGUMENT"),
                "Missing old_string.".to_string(),
                vec!["Provide old_string and new_string."],
                json!({}),
            );
        }
    };
    let new_string = match iter.next() {
        Some(Primitive::String(value)) => value,
        Some(_) => {
            return tool_response(
                false,
                "replace",
                Some("INVALID_ARGUMENT"),
                "new_string must be a string.".to_string(),
                vec!["Pass replacement text as new_string."],
                json!({}),
            );
        }
        None => {
            return tool_response(
                false,
                "replace",
                Some("INVALID_ARGUMENT"),
                "Missing new_string.".to_string(),
                vec!["Provide path, old_string, and new_string."],
                json!({}),
            );
        }
    };

    let max_replacements = match iter.next() {
        Some(Primitive::Number(value)) => {
            if value < 0.0 || value.fract() != 0.0 {
                return tool_response(
                    false,
                    "replace",
                    Some("INVALID_ARGUMENT"),
                    "max_replacements must be a non-negative integer.".to_string(),
                    vec!["Use 0 for unlimited replacements."],
                    json!({}),
                );
            }
            value as usize
        }
        Some(Primitive::Null) | None => 1,
        Some(_) => {
            return tool_response(
                false,
                "replace",
                Some("INVALID_ARGUMENT"),
                "max_replacements must be a number when provided.".to_string(),
                vec!["Use an integer value like 1, 2, or 0."],
                json!({}),
            );
        }
    };

    if iter.next().is_some() {
        return tool_response(
            false,
            "replace",
            Some("INVALID_ARGUMENT"),
            "Too many arguments passed to replace.".to_string(),
            vec!["Provide at most: workspace_dir, path, old_string, new_string, max_replacements."],
            json!({}),
        );
    }

    if !is_absolute_path(&file_path) {
        return tool_response(
            false,
            "replace",
            Some("INVALID_PATH"),
            "path must be absolute.".to_string(),
            vec!["Resolve the path using find_file before calling replace."],
            json!({ "path": file_path }),
        );
    }
    if !is_in_workspace(&workspace_dir, &file_path) {
        return tool_response(
            false,
            "replace",
            Some("OUTSIDE_WORKSPACE"),
            "path must be inside the workspace.".to_string(),
            vec!["Use a file path under the current workspace root."],
            json!({ "path": file_path, "workspace_dir": workspace_dir }),
        );
    }
    if old_string.is_empty() {
        return tool_response(
            false,
            "replace",
            Some("INVALID_ARGUMENT"),
            "old_string cannot be empty.".to_string(),
            vec!["Read the file and pass an exact non-empty old_string snippet."],
            json!({ "path": file_path }),
        );
    }

    let file_content = match std::fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            return tool_response(
                false,
                "replace",
                Some("READ_FAILED"),
                format!("Failed to read '{}': {}", file_path, e),
                vec!["Ensure the file exists and is readable."],
                json!({ "path": file_path }),
            );
        }
    };

    let matches_found = file_content.matches(&old_string).count();
    if matches_found == 0 {
        return tool_response(
            false,
            "replace",
            Some("NO_MATCH"),
            format!("No occurrences of old_string found in '{}'.", file_path),
            vec!["Read the file and copy an exact old_string snippet."],
            json!({
                "path": file_path,
                "matches_found": 0,
                "replacements_applied": 0
            }),
        );
    }

    let replacements_applied = if max_replacements == 0 {
        matches_found
    } else {
        matches_found.min(max_replacements)
    };
    let replaced_content = if max_replacements == 0 {
        file_content.replace(&old_string, &new_string)
    } else {
        file_content.replacen(&old_string, &new_string, max_replacements)
    };

    match std::fs::write(&file_path, replaced_content) {
        Ok(_) => tool_response(
            true,
            "replace",
            None,
            format!(
                "Applied {} replacement(s) in '{}'.",
                replacements_applied, file_path
            ),
            vec![],
            json!({
                "path": file_path,
                "matches_found": matches_found,
                "replacements_applied": replacements_applied
            }),
        ),
        Err(e) => tool_response(
            false,
            "replace",
            Some("WRITE_FAILED"),
            format!("Failed to write '{}': {}", file_path, e),
            vec!["Check file permissions and retry."],
            json!({
                "path": file_path,
                "matches_found": matches_found,
                "replacements_applied": 0
            }),
        ),
    }
}

#[rsl_native]
pub fn agent_search_workspace_tool(arguments: Vec<Primitive>) -> Primitive {
    let mut iter = arguments.into_iter();
    let workspace_dir = match iter.next() {
        Some(Primitive::String(value)) => value,
        _ => {
            return tool_response(
                false,
                "search_workspace",
                Some("INVALID_ARGUMENT"),
                "workspace_dir must be a string.".to_string(),
                vec!["Pass workspace_dir and pattern as strings."],
                json!({ "matches": [] }),
            );
        }
    };
    let pattern = match iter.next() {
        Some(Primitive::String(value)) => value,
        _ => {
            return tool_response(
                false,
                "search_workspace",
                Some("INVALID_ARGUMENT"),
                "pattern must be a string.".to_string(),
                vec!["Pass workspace_dir and pattern as strings."],
                json!({ "matches": [] }),
            );
        }
    };
    if iter.next().is_some() {
        return tool_response(
            false,
            "search_workspace",
            Some("INVALID_ARGUMENT"),
            "Too many arguments passed to search_workspace.".to_string(),
            vec!["Provide only workspace_dir and pattern."],
            json!({ "matches": [] }),
        );
    }
    if pattern.trim().is_empty() {
        return tool_response(
            false,
            "search_workspace",
            Some("INVALID_ARGUMENT"),
            "pattern cannot be empty.".to_string(),
            vec!["Pass a non-empty search pattern."],
            json!({ "matches": [] }),
        );
    }
    if which("rg").is_err() {
        return tool_response(
            false,
            "search_workspace",
            Some("COMMAND_NOT_FOUND"),
            "ripgrep (rg) is not available in PATH.".to_string(),
            vec!["Install ripgrep or use a different tool."],
            json!({ "matches": [] }),
        );
    }

    let output = match Command::new("rg")
        .arg("--json")
        .arg("--color=never")
        .arg("--smart-case")
        .arg("--max-count")
        .arg("200")
        .arg(pattern)
        .current_dir(&workspace_dir)
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            return tool_response(
                false,
                "search_workspace",
                Some("COMMAND_FAILED"),
                format!("Failed to execute rg: {}", err),
                vec!["Verify the workspace is accessible and retry."],
                json!({ "matches": [] }),
            );
        }
    };

    let status = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if status == 2 {
        let error_code = if stderr.contains("regex parse error") {
            "INVALID_REGEX"
        } else {
            "COMMAND_FAILED"
        };
        return tool_response(
            false,
            "search_workspace",
            Some(error_code),
            format!("search_workspace failed: {}", stderr.trim()),
            vec!["Use a simpler pattern first, then refine."],
            json!({ "matches": [] }),
        );
    }

    let mut matches = Vec::new();
    for line in stdout.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value["type"] != "match" {
            continue;
        }
        let path_text = value["data"]["path"]["text"].as_str().unwrap_or("");
        let line_number = value["data"]["line_number"].as_u64().unwrap_or(0);
        let text = value["data"]["lines"]["text"]
            .as_str()
            .unwrap_or("")
            .trim_end_matches('\n')
            .to_string();
        let absolute_path = Path::new(path_text);
        let absolute_path = if absolute_path.is_absolute() {
            absolute_path.to_path_buf()
        } else {
            Path::new(&workspace_dir).join(absolute_path)
        };

        let mut column = 1_u64;
        if let Some(submatch) = value["data"]["submatches"]
            .as_array()
            .and_then(|m| m.first())
        {
            column = submatch["start"].as_u64().unwrap_or(0) + 1;
        }

        matches.push(json!({
            "path": absolute_path.to_string_lossy().to_string(),
            "line": line_number,
            "column": column,
            "text": text
        }));
    }

    if matches.is_empty() {
        return tool_response(
            false,
            "search_workspace",
            Some("NO_RESULTS"),
            "No matches found.".to_string(),
            vec!["Try a broader pattern."],
            json!({ "matches": [] }),
        );
    }

    tool_response(
        true,
        "search_workspace",
        None,
        format!("Found {} match(es).", matches.len()),
        vec![],
        json!({ "matches": matches, "truncated": false }),
    )
}

#[rsl_native]
pub fn agent_find_file_tool(arguments: Vec<Primitive>) -> Primitive {
    let mut iter = arguments.into_iter();
    let workspace_dir = match iter.next() {
        Some(Primitive::String(value)) => value,
        _ => {
            return tool_response(
                false,
                "find_file",
                Some("INVALID_ARGUMENT"),
                "workspace_dir must be a string.".to_string(),
                vec!["Pass workspace_dir and pattern as strings."],
                json!({ "paths": [], "count": 0 }),
            );
        }
    };
    let pattern = match iter.next() {
        Some(Primitive::String(value)) => value,
        _ => {
            return tool_response(
                false,
                "find_file",
                Some("INVALID_ARGUMENT"),
                "pattern must be a string.".to_string(),
                vec!["Pass workspace_dir and pattern as strings."],
                json!({ "paths": [], "count": 0 }),
            );
        }
    };
    if iter.next().is_some() {
        return tool_response(
            false,
            "find_file",
            Some("INVALID_ARGUMENT"),
            "Too many arguments passed to find_file.".to_string(),
            vec!["Provide only workspace_dir and pattern."],
            json!({ "paths": [], "count": 0 }),
        );
    }
    if pattern.trim().is_empty() {
        return tool_response(
            false,
            "find_file",
            Some("INVALID_ARGUMENT"),
            "pattern cannot be empty.".to_string(),
            vec!["Pass a non-empty regex pattern."],
            json!({ "paths": [], "count": 0 }),
        );
    }
    if which("fd").is_err() {
        return tool_response(
            false,
            "find_file",
            Some("COMMAND_NOT_FOUND"),
            "fd is not available in PATH.".to_string(),
            vec!["Install fd-find or use another file-discovery tool."],
            json!({ "paths": [], "count": 0 }),
        );
    }

    let output = match Command::new("fd")
        .arg("--type")
        .arg("f")
        .arg("--strip-cwd-prefix")
        .arg("--full-path")
        .arg("--absolute-path")
        .arg(pattern)
        .current_dir(&workspace_dir)
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            return tool_response(
                false,
                "find_file",
                Some("COMMAND_FAILED"),
                format!("Failed to execute fd: {}", err),
                vec!["Verify the workspace is accessible and retry."],
                json!({ "paths": [], "count": 0 }),
            );
        }
    };

    let status = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if status != 0 {
        return tool_response(
            false,
            "find_file",
            Some("COMMAND_FAILED"),
            format!("find_file failed: {}", stderr.trim()),
            vec!["Check regex syntax and try a simpler pattern."],
            json!({ "paths": [], "count": 0 }),
        );
    }

    let paths: Vec<String> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect();

    if paths.is_empty() {
        return tool_response(
            false,
            "find_file",
            Some("NO_RESULTS"),
            "No files matched the pattern.".to_string(),
            vec!["Try a broader pattern like part of the filename."],
            json!({ "paths": [], "count": 0 }),
        );
    }

    tool_response(
        true,
        "find_file",
        None,
        format!("Found {} file(s).", paths.len()),
        vec![],
        json!({ "paths": paths, "count": paths.len() }),
    )
}

#[rsl_native]
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

#[rsl_native]
pub fn create_directory(arguments: Vec<Primitive>) -> Primitive {
    let path = args!(arguments; path: String);

    match std::fs::create_dir_all(&path) {
        Ok(_) => Primitive::Null,
        Err(e) => Primitive::Error(format!("creating directory '{}': {}", path, e)),
    }
}

#[rsl_native]
pub fn read_file(arguments: Vec<Primitive>) -> Primitive {
    let path = args!(arguments; path: String);
    let content = std::fs::read_to_string(path).unwrap();
    Primitive::String(content)
}

#[rsl_native]
pub fn get_env_var(arguments: Vec<Primitive>) -> Primitive {
    let key = args!(arguments; key: String);
    match std::env::var(key) {
        Ok(value) => Primitive::String(value),
        Err(std::env::VarError::NotPresent) => Primitive::Null,
        Err(err) => Primitive::Error(format!("reading env var failed: {}", err)),
    }
}

#[rsl_native]
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

#[rsl_native]
pub fn command_exists(arguments: Vec<Primitive>) -> Primitive {
    let command = args!(arguments; command: String);
    Primitive::Boolean(which(command).is_ok())
}

#[rsl_native]
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

#[rsl_native]
pub fn join_path(arguments: Vec<Primitive>) -> Primitive {
    let (base, segment) = args!(arguments; base: String, segment: String);

    let mut path = PathBuf::from(base);
    path.push(segment);

    Primitive::String(path.to_string_lossy().to_string())
}

#[rsl_native]
pub fn parent_path(arguments: Vec<Primitive>) -> Primitive {
    let path = args!(arguments; path: String);

    let path = PathBuf::from(path);
    let parent = path.parent().map(PathBuf::from).unwrap_or(path);

    Primitive::String(parent.to_string_lossy().to_string())
}
