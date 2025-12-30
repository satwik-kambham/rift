use std::process::Stdio;

use tokio::{io::AsyncWriteExt, process::Command, sync::mpsc::Sender};

use crate::state::EditorState;

use super::{AsyncError, AsyncResult};

#[derive(Debug)]
pub struct ProgramArgs {
    pub program: String,
    pub args: Vec<String>,
}

pub fn run_command(
    program_args: ProgramArgs,
    callback: fn(Result<String, AsyncError>, state: &mut EditorState),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
    current_dir: String,
) {
    let ProgramArgs { program, args } = program_args;
    let program_for_err = program.clone();
    let args_for_err = args.clone();

    let missing_program_result = if which::which(&program).is_err() {
        Some(AsyncError::Process {
            program,
            args,
            status: None,
            stderr: String::new(),
            message: "command not found".to_string(),
        })
    } else {
        None
    };

    if let Some(err) = missing_program_result {
        rt.spawn(async move {
            sender
                .send(AsyncResult {
                    result: Err(err),
                    callback,
                })
                .await
                .unwrap();
        });
        return;
    }

    rt.spawn(async move {
        let program_err = program_for_err.clone();
        let args_err = args_for_err.clone();

        let result = async {
            let output = Command::new(&program_for_err)
                .args(&args_for_err)
                .current_dir(&current_dir)
                .output()
                .await
                .map_err(|err| AsyncError::Process {
                    program: program_for_err.clone(),
                    args: args_for_err.clone(),
                    status: None,
                    stderr: String::new(),
                    message: err.to_string(),
                })?;

            let status_code = output.status.code();
            if !output.status.success() {
                return Err(AsyncError::Process {
                    program: program_for_err,
                    args: args_for_err,
                    status: status_code,
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    message: "process exited with non-zero status".to_string(),
                });
            }

            String::from_utf8(output.stdout).map_err(|err| AsyncError::Process {
                program: program_err,
                args: args_err,
                status: status_code,
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                message: err.to_string(),
            })
        }
        .await;

        sender.send(AsyncResult { result, callback }).await.unwrap();
    });
}

pub fn run_piped_commands(
    program_args: Vec<ProgramArgs>,
    callback: fn(Result<String, AsyncError>, state: &mut EditorState),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
    current_dir: String,
) {
    let mut program_args = program_args;

    rt.spawn(async move {
        for program in &program_args {
            if which::which(&program.program).is_err() {
                sender
                    .send(AsyncResult {
                        result: Err(AsyncError::Process {
                            program: program.program.clone(),
                            args: program.args.clone(),
                            status: None,
                            stderr: String::new(),
                            message: "command not found".to_string(),
                        }),
                        callback,
                    })
                    .await
                    .unwrap();
                return;
            }
        }

        let mut previous_result: Option<Vec<u8>> = None;

        for program in program_args.drain(..) {
            let mut command = Command::new(&program.program);
            command
                .args(&program.args)
                .current_dir(&current_dir)
                .stdout(Stdio::piped());

            if let Some(output) = previous_result {
                command.stdin(Stdio::piped());
                let mut child = match command.spawn() {
                    Ok(child) => child,
                    Err(err) => {
                        sender
                            .send(AsyncResult {
                                result: Err(AsyncError::Process {
                                    program: program.program,
                                    args: program.args,
                                    status: None,
                                    stderr: String::new(),
                                    message: format!("Failed to start command: {}", err),
                                }),
                                callback,
                            })
                            .await
                            .unwrap();
                        return;
                    }
                };
                if let Some(mut stdin) = child.stdin.take()
                    && let Err(err) = stdin.write_all(&output).await
                {
                    sender
                        .send(AsyncResult {
                            result: Err(AsyncError::Process {
                                program: program.program,
                                args: program.args,
                                status: None,
                                stderr: String::new(),
                                message: format!("Failed to write to stdin: {}", err),
                            }),
                            callback,
                        })
                        .await
                        .unwrap();
                    return;
                }
                let output = match child.wait_with_output().await {
                    Ok(output) => output,
                    Err(err) => {
                        sender
                            .send(AsyncResult {
                                result: Err(AsyncError::Process {
                                    program: program.program,
                                    args: program.args,
                                    status: None,
                                    stderr: String::new(),
                                    message: format!("Failed to wait for command: {}", err),
                                }),
                                callback,
                            })
                            .await
                            .unwrap();
                        return;
                    }
                };

                let status_code = output.status.code();
                if !output.status.success() {
                    sender
                        .send(AsyncResult {
                            result: Err(AsyncError::Process {
                                program: program.program,
                                args: program.args,
                                status: status_code,
                                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                                message: "process exited with non-zero status".to_string(),
                            }),
                            callback,
                        })
                        .await
                        .unwrap();
                    return;
                }

                previous_result = Some(output.stdout);
            } else {
                let output = match command.output().await {
                    Ok(output) => output,
                    Err(err) => {
                        sender
                            .send(AsyncResult {
                                result: Err(AsyncError::Process {
                                    program: program.program,
                                    args: program.args,
                                    status: None,
                                    stderr: String::new(),
                                    message: format!("Failed to run command: {}", err),
                                }),
                                callback,
                            })
                            .await
                            .unwrap();
                        return;
                    }
                };

                let status_code = output.status.code();
                if !output.status.success() {
                    sender
                        .send(AsyncResult {
                            result: Err(AsyncError::Process {
                                program: program.program,
                                args: program.args,
                                status: status_code,
                                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                                message: "process exited with non-zero status".to_string(),
                            }),
                            callback,
                        })
                        .await
                        .unwrap();
                    return;
                }

                previous_result = Some(output.stdout);
            }
        }

        let result = previous_result
            .ok_or_else(|| AsyncError::Process {
                program: String::new(),
                args: vec![],
                status: None,
                stderr: String::new(),
                message: "no commands executed".to_string(),
            })
            .and_then(|output| {
                String::from_utf8(output).map_err(|err| AsyncError::Process {
                    program: String::new(),
                    args: vec![],
                    status: None,
                    stderr: String::new(),
                    message: err.to_string(),
                })
            });

        sender.send(AsyncResult { result, callback }).await.unwrap();
    });
}
