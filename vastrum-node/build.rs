fn main() {
    //always rebuild web-client to ensure all changes are up to date always
    println!("cargo:rerun-if-changed=ALWAYS_REBUILD");
    let script = match std::env::var("PROFILE").unwrap().as_str() {
        "release" => "build:prod",
        _ => "build",
    };

    let status = std::process::Command::new("npm")
        .args(["run", script])
        .current_dir("../web-client/app")
        .env("RUSTFLAGS", "")
        .env("CARGO_ENCODED_RUSTFLAGS", "")
        .status()
        .expect("failed to run npm build in web-client/");
    assert!(status.success(), "web-client npm build failed");
}
