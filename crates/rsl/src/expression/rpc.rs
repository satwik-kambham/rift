use tarpc::context;

use crate::RSL;
use crate::errors::RuntimeError;
use crate::primitive::Primitive;
use crate::std_lib::args;
use crate::token::Span;

pub(super) fn execute_rpc_call(
    identifier: &str,
    parameters: Vec<Primitive>,
    rsl: &mut RSL,
    span: Span,
) -> Result<Primitive, RuntimeError> {
    match identifier {
        "log"
        | "openFile"
        | "setActiveBuffer"
        | "getActiveBuffer"
        | "listBuffers"
        | "getActions"
        | "getReferences"
        | "getDefinitions"
        | "getWorkspaceDiagnostics"
        | "getViewportSize"
        | "selectRange"
        | "registerGlobalKeybind"
        | "registerBufferKeybind"
        | "registerBufferInputHook"
        | "createSpecialBuffer"
        | "setBufferContent"
        | "getBufferInput"
        | "setBufferInput"
        | "setSearchQuery"
        | "getWorkspaceDir"
        | "runAction"
        | "tts" => {}
        _ => {
            return Err(RuntimeError::new(
                format!("RPC function '{}' does not exist", identifier),
                span,
            ));
        }
    }

    Ok(rsl.rt_handle.block_on(async {
        let ctx = context::Context::current();
        let client = &rsl.rift_rpc_client;
        let rpc_err = |e: tarpc::client::RpcError| -> Primitive {
            Primitive::Error(format!("RPC call '{identifier}' failed: {e}"))
        };

        match identifier {
            "log" => {
                let message = parameters
                    .iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<_>>()
                    .join(" ");
                if let Err(e) = client.rlog(ctx, message).await {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            "openFile" => {
                let path = args!(parameters; path: String);
                if let Err(e) = client.open_file(ctx, path).await {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            "setActiveBuffer" => {
                let buffer_id = args!(parameters; buffer_id: Number);
                if let Err(e) = client.set_active_buffer(ctx, buffer_id as u32).await {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            "getActiveBuffer" => match client.get_active_buffer(ctx).await {
                Ok(Some(buffer_id)) => Primitive::Number(buffer_id as f32),
                Ok(None) => Primitive::Null,
                Err(e) => rpc_err(e),
            },
            "listBuffers" => match client.list_buffers(ctx).await {
                Ok(buffers) => Primitive::String(buffers),
                Err(e) => rpc_err(e),
            },
            "getActions" => match client.get_actions(ctx).await {
                Ok(actions) => Primitive::String(actions),
                Err(e) => rpc_err(e),
            },
            "getReferences" => match client.get_references(ctx).await {
                Ok(references) => Primitive::String(references),
                Err(e) => rpc_err(e),
            },
            "getDefinitions" => match client.get_definitions(ctx).await {
                Ok(definitions) => Primitive::String(definitions),
                Err(e) => rpc_err(e),
            },
            "getWorkspaceDiagnostics" => match client.get_workspace_diagnostics(ctx).await {
                Ok(diagnostics) => Primitive::String(diagnostics),
                Err(e) => rpc_err(e),
            },
            "getViewportSize" => match client.get_viewport_size(ctx).await {
                Ok(size) => Primitive::String(size),
                Err(e) => rpc_err(e),
            },
            "selectRange" => {
                let selection = args!(parameters; selection: String);
                if let Err(e) = client.select_range(ctx, selection).await {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            "registerGlobalKeybind" => {
                let (definition, function_id) =
                    args!(parameters; definition: String, function_id: Function);
                if let Err(e) = client
                    .register_global_keybind(ctx, definition, function_id)
                    .await
                {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            "registerBufferKeybind" => {
                let (buffer_id, definition, function_id) = args!(
                    parameters;
                    buffer_id: Number,
                    definition: String,
                    function_id: Function
                );
                if let Err(e) = client
                    .register_buffer_keybind(ctx, buffer_id as u32, definition, function_id)
                    .await
                {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            "registerBufferInputHook" => {
                let (buffer_id, function_id) =
                    args!(parameters; buffer_id: Number, function_id: Function);
                if let Err(e) = client
                    .register_buffer_input_hook(ctx, buffer_id as u32, function_id)
                    .await
                {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            "createSpecialBuffer" => {
                let display_name = if parameters.is_empty() {
                    "".to_string()
                } else {
                    args!(parameters; display_name: String)
                };
                match client.create_special_buffer(ctx, display_name).await {
                    Ok(buffer_id) => Primitive::Number(buffer_id as f32),
                    Err(e) => rpc_err(e),
                }
            }
            "setBufferContent" => {
                let (buffer_id, content) = args!(parameters; buffer_id: Number, content: String);
                if let Err(e) = client
                    .set_buffer_content(ctx, buffer_id as u32, content)
                    .await
                {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            "getBufferInput" => {
                let buffer_id = args!(parameters; buffer_id: Number);
                match client.get_buffer_input(ctx, buffer_id as u32).await {
                    Ok(input) => Primitive::String(input),
                    Err(e) => rpc_err(e),
                }
            }
            "setBufferInput" => {
                let (buffer_id, input) = args!(parameters; buffer_id: Number, input: String);
                if let Err(e) = client.set_buffer_input(ctx, buffer_id as u32, input).await {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            "setSearchQuery" => {
                let query = args!(parameters; query: String);
                if let Err(e) = client.set_search_query(ctx, query).await {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            "getWorkspaceDir" => match client.get_workspace_dir(ctx).await {
                Ok(workspace_dir) => Primitive::String(workspace_dir),
                Err(e) => rpc_err(e),
            },
            "runAction" => {
                let action = args!(parameters; action: String);
                match client.run_action(ctx, action).await {
                    Ok(result) => Primitive::String(result),
                    Err(e) => rpc_err(e),
                }
            }
            "tts" => {
                let text = args!(parameters; text: String);
                if let Err(e) = client.tts(ctx, text).await {
                    return rpc_err(e);
                }
                Primitive::Null
            }
            // SAFETY: unreachable because unknown identifiers are rejected above
            _ => unreachable!(),
        }
    }))
}
