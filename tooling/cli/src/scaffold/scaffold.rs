use rust_embed::RustEmbed;
use std::path::Path;

#[cfg(not(madsim))]
#[derive(RustEmbed)]
#[folder = "scaffolds/site/"]
struct SiteTemplate;

#[cfg(not(madsim))]
#[derive(RustEmbed)]
#[folder = "scaffolds/eth_dapp/"]
struct EthDappTemplate;

pub fn initialize_new_project(name: String, template: String) {
    if !name.chars().next().map_or(false, |c| c.is_ascii_lowercase()) {
        eprintln!("Error: name must start with a lowercase letter");
        return;
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        eprintln!("Error: name must contain only lowercase letters, digits, and hyphens");
        return;
    }

    let target = Path::new(&name);
    if target.exists() {
        eprintln!("Error: directory '{}' already exists", name);
        return;
    }

    let name_cap = capitalize(&name);

    #[cfg(not(madsim))]
    match template.as_str() {
        "site" => extract_template::<SiteTemplate>(target, &name, &name_cap),
        "eth_dapp" => extract_template::<EthDappTemplate>(target, &name, &name_cap),
        _ => {
            eprintln!("Error: unknown template '{}'. Use 'site' or 'eth_dapp'", template);
            return;
        }
    }

    #[cfg(madsim)]
    {
        eprintln!("Error: scaffold is not available in madsim builds");
        return;
    }

    println!("Created project '{name}' with template '{template}'");
    println!("\nNext steps:");
    println!("  1. cd {name} && vastrum-cli run-dev");
    println!();
    println!("This starts a local node, opens the browser, and deploys");
    println!("your contract. The site will load once deployment completes.");
    println!();
    println!("Docs: docs.vastrum.org");
}

#[cfg(not(madsim))]
fn extract_template<T: RustEmbed>(target_dir: &Path, name: &str, name_cap: &str) {
    let name_underscore = name.replace('-', "_");
    for file_path in T::iter() {
        let content = T::get(&file_path).unwrap();
        let content_str = std::str::from_utf8(content.data.as_ref()).unwrap();
        let processed = content_str
            .replace("{{name_underscore}}", &name_underscore)
            .replace("{{name}}", name)
            .replace("{{Name}}", name_cap)
            .replace("{{git_repo}}", "https://github.com/vastrum/vastrum-monorepo")
            .replace("{{react_lib}}", "^0.1.0");

        // RustEmbed skips dotfiles by default, have to replace dot_ with .
        // Cargo_toml avoids Cargo parsing template files in git checkouts
        let out_name = file_path
            .as_ref()
            .replace("dot_gitignore", ".gitignore")
            .replace("Cargo_toml", "Cargo.toml");
        let out_path = target_dir.join(&out_name);
        std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();
        std::fs::write(&out_path, processed).unwrap();
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => return String::new(),
        Some(c) => return c.to_uppercase().to_string() + chars.as_str(),
    }
}
