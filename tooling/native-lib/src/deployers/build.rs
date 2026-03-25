use std::process::Command;

pub fn run(cmd: &str, dir: &str) {
    let mut parts = cmd.split_whitespace();
    let program = parts.next().expect("empty command");
    let args: Vec<&str> = parts.collect();
    let output = Command::new(program)
        .args(&args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {cmd}: {e}"));
    assert!(
        output.status.success(),
        "{cmd} failed in {dir}:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Build a contract crate for wasm32 and copy the artifact to {out_dir}/contract.wasm.
pub fn build_contract(contract_dir: &str, out_dir: &str) {
    let output = Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release", "--message-format=json"])
        .current_dir(contract_dir)
        .output()
        .expect("failed to build contract wasm");
    assert!(
        output.status.success(),
        "contract wasm build failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut wasm_path = None;
    for line in stdout.lines() {
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        if msg["reason"] != "compiler-artifact" {
            continue;
        }
        let Some(filenames) = msg["filenames"].as_array() else { continue };
        for f in filenames {
            if let Some(s) = f.as_str().filter(|s| s.ends_with(".wasm")) {
                wasm_path = Some(s.to_owned());
            }
        }
    }
    let wasm_path = wasm_path.expect("no wasm artifact in cargo output");

    let out_path = std::path::Path::new(out_dir);
    std::fs::create_dir_all(out_path).unwrap();
    std::fs::copy(&wasm_path, out_path.join("contract.wasm")).unwrap();
}
