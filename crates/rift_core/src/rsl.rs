use std::collections::HashMap;
use std::path::PathBuf;

use include_dir::{Dir, include_dir};
use tokio::sync::mpsc;

use rsl::RSL;

use crate::{
    actions::{Action, perform_action},
    state::EditorState,
};

static RSL_MODULES: Dir = include_dir!("$CARGO_MANIFEST_DIR/modules");

fn walk_dir<'a>(dir: &Dir<'a>, map: &mut HashMap<&'a str, &'a str>) {
    for file in dir.files() {
        let path = file.path().to_str().unwrap();
        let contents = file.contents_utf8().unwrap();
        map.insert(path, contents);
    }
    for subdir in dir.dirs() {
        walk_dir(subdir, map);
    }
}

fn embedded_text_files() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();

    for file in RSL_MODULES.files() {
        let path = file.path().to_str().unwrap();
        let contents = file.contents_utf8().unwrap();
        map.insert(path, contents);
    }

    walk_dir(&RSL_MODULES, &mut map);
    map
}

pub(crate) fn start_rsl_interpreter(
    initial_folder: String,
    rpc_client_transport: tarpc::transport::channel::UnboundedChannel<
        tarpc::Response<rift_rpc::RiftRPCResponse>,
        tarpc::ClientMessage<rift_rpc::RiftRPCRequest>,
    >,
) -> mpsc::Sender<String> {
    let (rsl_sender, mut rsl_reciever) = mpsc::channel::<String>(32);

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(err) => {
                tracing::error!(%err, "Failed to build RSL runtime; RSL scripts disabled");
                return;
            }
        };

        let rsl_modules = embedded_text_files();
        let mut rsl_interpreter = RSL::new(
            Some(PathBuf::from(&initial_folder)),
            rt.handle().clone(),
            rsl_modules,
            rpc_client_transport,
        );
        while let Some(source) = rsl_reciever.blocking_recv() {
            if let Err(e) = rsl_interpreter.run(source) {
                tracing::error!(%e, "RSL execution error");
            }
        }
    });

    rsl_sender
}

pub fn initialize_rsl(state: &mut EditorState) {
    let init_module = include_str!("../modules/init.rsl").to_string();
    perform_action(Action::RunSource(init_module), state);
}
