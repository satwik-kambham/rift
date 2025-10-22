pub mod array;
pub mod environment;
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

use crate::environment::Environment;

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
        environment.register_native_function("getRequest", std_lib::web_requests::get_request);
        environment.register_native_function("postRequest", std_lib::web_requests::post_request);
        environment.register_native_function(
            "postRequestWithBearerToken",
            std_lib::web_requests::post_request_with_bearer_token,
        );

        #[cfg(feature = "rift_rpc")]
        let rpc_client =
            rift_rpc::RiftRPCClient::new(tarpc::client::Config::default(), rpc_client_transport);
        #[cfg(feature = "rift_rpc")]
        let rpc_client = rt_handle.block_on(async { rpc_client.spawn() });

        Self {
            environment: Rc::new(environment),
            rt_handle,
            working_dir: working_dir.unwrap_or(std::env::current_dir().unwrap()),
            #[cfg(feature = "rift_rpc")]
            rift_rpc_client: rpc_client,
        }
    }

    pub fn run(&mut self, source: String) {
        self.run_with_environment(source, self.environment.clone());
    }

    pub fn run_with_environment(&mut self, source: String, environment: Rc<Environment>) {
        let mut scanner = crate::scanner::Scanner::new(source);
        let tokens = scanner.scan();

        let mut parser = crate::parser::Parser::new(tokens.clone());
        let statements = parser.parse();

        let mut interpreter =
            crate::interpreter::Interpreter::with_environment(statements, environment);
        interpreter.interpret(self);
    }

    pub fn get_package_code(&self, package_name: &str) -> String {
        let candidate = self.working_dir.join(package_name);
        if candidate.is_file() {
            let source = std::fs::read_to_string(candidate).unwrap();
            return source;
        }
        panic!("Package not found at {:?}", candidate)
    }
}
