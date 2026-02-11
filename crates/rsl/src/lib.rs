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

extern crate self as rsl;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crate::{
    environment::{Environment, NativeFunction},
    errors::RSLError,
};
use anyhow::Context;

pub struct NativeFnRegistration {
    pub name: &'static str,
    pub func: NativeFunction,
}

inventory::collect!(NativeFnRegistration);

#[macro_export]
macro_rules! submit_native_function {
    ($name:expr, $func:path) => {
        inventory::submit!(rsl::NativeFnRegistration {
            name: $name,
            func: $func,
        });
    };
}

pub fn register_native_functions(environment: &Environment) {
    for registration in inventory::iter::<NativeFnRegistration> {
        environment.register_native_function(registration.name, registration.func);
    }
}

pub struct RSL<'a> {
    pub environment: Rc<Environment>,
    pub rt_handle: tokio::runtime::Handle,
    working_dir: PathBuf,
    modules: HashMap<&'a str, &'a str>,
    cached_modules: HashMap<String, Rc<RefCell<table::Table>>>,

    #[cfg(feature = "rift_rpc")]
    pub rift_rpc_client: rift_rpc::RiftRPCClient,
}

impl<'a> RSL<'a> {
    pub fn new(
        working_dir: Option<PathBuf>,
        rt_handle: tokio::runtime::Handle,
        modules: HashMap<&'a str, &'a str>,
        #[cfg(feature = "rift_rpc")]
        rpc_client_transport: tarpc::transport::channel::UnboundedChannel<
            tarpc::Response<rift_rpc::RiftRPCResponse>,
            tarpc::ClientMessage<rift_rpc::RiftRPCRequest>,
        >,
    ) -> Self {
        let environment = Environment::new(None);

        register_native_functions(&environment);

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
            modules,
            cached_modules: HashMap::new(),
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
        interpreter.interpret(self)?;

        Ok(())
    }

    pub fn get_package_code(&self, package_name: &str) -> anyhow::Result<String> {
        if let Some(source) = self.modules.get(package_name) {
            return Ok(source.to_string());
        }

        let candidate = self.working_dir.join(package_name);
        if candidate.is_file() {
            let source = std::fs::read_to_string(&candidate)
                .with_context(|| format!("reading package at {}", candidate.display()))?;
            return Ok(source);
        }
        anyhow::bail!("Package not found at {:?}", candidate)
    }

    pub fn cached_import(
        &mut self,
        package_name: &String,
        span: token::Span,
    ) -> Result<primitive::Primitive, errors::RuntimeError> {
        if let Some(exported_values) = self.cached_modules.get(package_name) {
            return Ok(primitive::Primitive::Table(exported_values.clone()));
        }

        match self.get_package_code(package_name) {
            Ok(source) => {
                let local_environment = Rc::new(Environment::new(Some(self.environment.clone())));
                self.run_with_environment(source, local_environment.clone())
                    .map_err(|e| {
                        errors::RuntimeError::new(
                            format!("Failed to import package {}: {}", package_name, e),
                            span,
                        )
                    })?;
                let exported_values = local_environment.get_exported_values();
                let exported_values = Rc::new(RefCell::new(exported_values));
                self.cached_modules
                    .insert(package_name.clone(), exported_values.clone());
                let exported_values = primitive::Primitive::Table(exported_values);
                Ok(exported_values)
            }
            Err(err) => {
                eprintln!("Failed to import package {}: {}", package_name, err);
                Ok(primitive::Primitive::Null)
            }
        }
    }
}
