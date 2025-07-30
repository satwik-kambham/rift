use std::{collections::HashMap, process::Stdio};

use tokio::{io::AsyncWriteExt, process::Command, sync::mpsc::Sender};

use crate::{buffer::instance::Language, lsp::client::LSPClientHandle, state::EditorState};

use super::AsyncResult;

#[derive(Debug)]
pub struct ProgramArgs {
    pub program: String,
    pub args: Vec<String>,
}

pub fn run_command(
    program_args: ProgramArgs,
    callback: fn(
        String,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
    current_dir: String,
) {
    if which::which(&program_args.program).is_err() {
        return;
    }

    rt.spawn(async move {
        let result = String::from_utf8(
            Command::new(program_args.program)
                .args(program_args.args)
                .current_dir(current_dir)
                .output()
                .await
                .unwrap()
                .stdout,
        )
        .unwrap();
        sender.send(AsyncResult { result, callback }).await.unwrap();
    });
}

pub fn run_piped_commands(
    program_args: Vec<ProgramArgs>,
    callback: fn(
        String,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
    current_dir: String,
) {
    for program in &program_args {
        if which::which(&program.program).is_err() {
            return;
        }
    }

    rt.spawn(async move {
        let mut previous_result: Option<Vec<u8>> = None;

        for program in program_args {
            let mut command = Command::new(&program.program);
            command.args(&program.args).current_dir(&current_dir).stdout(Stdio::piped());

            if let Some(output) = previous_result {
                command.stdin(Stdio::piped());
                let mut child = command.spawn().expect("Failed to start command");
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(&output).await.unwrap();
                }
                previous_result = Some(child.wait_with_output().await.unwrap().stdout);
            } else {
                previous_result = Some(
                    command
                        .output()
                        .await
                        .expect("Failed to run command")
                        .stdout,
                );
            }
        }

        let result = String::from_utf8(previous_result.unwrap()).unwrap();

        sender.send(AsyncResult { result, callback }).await.unwrap();
    });
}
