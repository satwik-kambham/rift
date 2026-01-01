pub mod array;
pub mod environment;
pub mod errors;
pub mod expression;
pub mod interpreter;
pub mod operator;
pub mod parser;
pub mod primitive;
pub mod scanner;
pub mod statement;
pub mod std_lib;
pub mod table;
pub mod token;

use std::path::PathBuf;
use std::rc::Rc;

use crate::{environment::Environment, errors::RSLError};
use anyhow::Context;

pub struct RSL {
    pub environment: Rc<Environment>,
    pub rt_handle: tokio::runtime::Handle,
    working_dir: PathBuf,

    #[cfg(feature = "rift_rpc")]
    pub rift_rpc_client: rift_rpc::RiftRPCClient,
}

impl RSL {
    pub fn new(
        working_dir: Option<PathBuf>,
        rt_handle: tokio::runtime::Handle,
        #[cfg(feature = "rift_rpc")]
        rpc_client_transport: tarpc::transport::channel::UnboundedChannel<
            tarpc::Response<rift_rpc::RiftRPCResponse>,
            tarpc::ClientMessage<rift_rpc::RiftRPCRequest>,
        >,
    ) -> Self {
        let environment = Environment::new(None);

        environment.register_native_function("print", std_lib::print);
        environment.register_native_function("toString", std_lib::to_string);
        environment.register_native_function("toJson", std_lib::to_json);
        environment.register_native_function("fromJson", std_lib::from_json);
        environment.register_native_function("createArray", std_lib::array::create_array);
        environment.register_native_function("arrayLen", std_lib::array::array_len);
        environment.register_native_function("arrayGet", std_lib::array::array_get);
        environment.register_native_function("arraySet", std_lib::array::array_set);
        environment.register_native_function("arrayPushBack", std_lib::array::array_push_back);
        environment.register_native_function("arrayRemove", std_lib::array::array_remove);
        environment.register_native_function("arrayPopBack", std_lib::array::array_pop_back);
        environment.register_native_function("createTable", std_lib::table::create_table);
        environment.register_native_function("tableSet", std_lib::table::table_set);
        environment.register_native_function("tableGet", std_lib::table::table_get);
        environment.register_native_function("tableKeys", std_lib::table::table_keys);
        environment
            .register_native_function("stringSplitLines", std_lib::string::string_split_lines);
        environment.register_native_function("stringLen", std_lib::string::string_len);
        environment.register_native_function("stringContains", std_lib::string::string_contains);
        environment.register_native_function("stringToLower", std_lib::string::string_to_lower);
        environment.register_native_function("stringWidth", std_lib::string::string_width);
        environment.register_native_function(
            "stringTruncateWidth",
            std_lib::string::string_truncate_width,
        );
        environment.register_native_function("getRequest", std_lib::web_requests::get_request);
        environment.register_native_function("postRequest", std_lib::web_requests::post_request);
        environment.register_native_function(
            "postRequestWithBearerToken",
            std_lib::web_requests::post_request_with_bearer_token,
        );
        environment.register_native_function("readFile", std_lib::io::read_file);
        environment.register_native_function("getEnvVar", std_lib::io::get_env_var);
        environment.register_native_function("runShellCommand", std_lib::io::run_shell_command);
        environment.register_native_function("commandExists", std_lib::io::command_exists);
        environment.register_native_function("agentReadFile", std_lib::io::agent_read_file);
        environment.register_native_function("agentWriteFile", std_lib::io::agent_write_file);
        environment.register_native_function("agentReplace", std_lib::io::agent_replace);
        environment.register_native_function("createBlankFile", std_lib::io::create_blank_file);
        environment.register_native_function("createDirectory", std_lib::io::create_directory);
        environment.register_native_function("listDir", std_lib::io::list_dir);
        environment.register_native_function("joinPath", std_lib::io::join_path);
        environment.register_native_function("parentPath", std_lib::io::parent_path);

        #[cfg(feature = "rift_rpc")]
        let rpc_client =
            rift_rpc::RiftRPCClient::new(tarpc::client::Config::default(), rpc_client_transport);
        #[cfg(feature = "rift_rpc")]
        let rpc_client = rt_handle.block_on(async { rpc_client.spawn() });

        let working_dir = working_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));

        Self {
            environment: Rc::new(environment),
            rt_handle,
            working_dir,
            #[cfg(feature = "rift_rpc")]
            rift_rpc_client: rpc_client,
        }
    }

    pub fn run(&mut self, source: String) -> Result<(), RSLError> {
        self.run_with_environment(source, self.environment.clone())?;
        Ok(())
    }

    pub fn run_with_environment(
        &mut self,
        source: String,
        environment: Rc<Environment>,
    ) -> Result<(), RSLError> {
        let mut scanner = crate::scanner::Scanner::new(source);
        let tokens = scanner.scan()?;

        let mut parser = crate::parser::Parser::new(tokens);
        let statements = parser.parse()?;

        let mut interpreter =
            crate::interpreter::Interpreter::with_environment(statements, environment);
        interpreter.interpret(self);

        Ok(())
    }

    pub fn get_package_code(&self, package_name: &str) -> anyhow::Result<String> {
        let candidate = self.working_dir.join(package_name);
        if candidate.is_file() {
            let source = std::fs::read_to_string(&candidate)
                .with_context(|| format!("reading package at {}", candidate.display()))?;
            return Ok(source);
        }
        anyhow::bail!("Package not found at {:?}", candidate)
    }
}
