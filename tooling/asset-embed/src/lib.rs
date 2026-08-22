use std::path::{Path, PathBuf};

pub struct EmbeddedAsset {
    pub path: &'static str,
    pub data: &'static [u8],
}

pub struct EmbedRequest<'a> {
    pub folder: &'a str,
    pub static_name: &'a str,
    pub out_file: &'a str,
}

pub fn generate(request: EmbedRequest) {
    let root =
        Path::new(request.folder).canonicalize().unwrap_or_else(|error| panic!("cannot embed "));
    println!("cargo:rerun-if-changed={}", root.display());

    let mut files = Vec::new();
    collect_files(&root, &mut files);
    files.sort();

    let table = render_table(request.static_name, &root, &files);
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo for build scripts");
    let out_path = Path::new(&out_dir).join(request.out_file);
    std::fs::write(&out_path, table)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", out_path.display()));
}

fn render_table(static_name: &str, root: &Path, files: &[PathBuf]) -> String {
    let mut generated =
        format!("pub static {static_name}: &[::vastrum_asset_embed::EmbeddedAsset] = &[\n");
    for path in files {
        let relative = path
            .strip_prefix(root)
            .expect("collected path is inside root")
            .to_string_lossy()
            .replace('\\', "/");
        generated.push_str(&format!(
            "    ::vastrum_asset_embed::EmbeddedAsset {{ path: {relative:?}, data: include_bytes!({:?}) }},\n",
            path.to_string_lossy()
        ));
    }
    generated.push_str("];\n");
    generated
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
    {
        let path = entry.expect("failed to read directory entry").path();
        let name = path.file_name().expect("directory entry has a name").to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_files(&path, out);
        } else {
            out.push(path);
        }
    }
}
