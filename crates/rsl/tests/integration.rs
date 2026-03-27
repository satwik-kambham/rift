// Test script index: tests/scripts/TEST_INDEX.md

use std::collections::HashMap;
use std::path::PathBuf;

fn run_rsl_script(path: &PathBuf) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let mut rsl = rsl::RSL::new(None, rt.handle().clone(), HashMap::new());
    rsl.run(source)
        .unwrap_or_else(|e| panic!("{} failed: {e}", path.display()));
}

#[test]
fn rsl_scripts() {
    let scripts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/scripts");
    let mut scripts: Vec<_> = std::fs::read_dir(&scripts_dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", scripts_dir.display()))
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension().is_some_and(|ext| ext == "rsl") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    scripts.sort();

    assert!(!scripts.is_empty(), "no .rsl test scripts found");

    for script in &scripts {
        let name = script.file_stem().unwrap().to_string_lossy();
        println!("running {name}...");
        run_rsl_script(script);
    }
}
