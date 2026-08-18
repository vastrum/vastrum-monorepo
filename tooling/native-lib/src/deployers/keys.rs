use std::path::PathBuf;

pub fn write_deploy_keys(app: &str, contents: &str) {
    let deploy_dir = repo_root().join("deploy");
    std::fs::create_dir_all(&deploy_dir).expect("failed to create deploy dir");
    let path = deploy_dir.join(format!("{app}.keys"));
    std::fs::write(&path, contents)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
    println!("secrets written to {}", path.display());
}

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("cwd");
    loop {
        if dir.join("Cargo.toml").is_file() && dir.join("apps").is_dir() {
            return dir;
        }
        if !dir.pop() {
            panic!("");
        }
    }
}
