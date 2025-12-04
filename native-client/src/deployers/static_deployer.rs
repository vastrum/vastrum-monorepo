fn find_html_files(base_dir: &Path) -> Result<Vec<(PathBuf, PathBuf)>, std::io::Error> {
    let mut html_files = Vec::new();

    // Recursively walk through directories
    fn walk_dir(
        dir: &Path,
        base: &Path,
        files: &mut Vec<(PathBuf, PathBuf)>,
    ) -> Result<(), std::io::Error> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    walk_dir(&path, base, files)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("html") {
                    let absolute = path.canonicalize()?;
                    let relative = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
                    let relative_without_ext = relative.with_extension("");

                    files.push((relative_without_ext, absolute));
                }
            }
        }
        Ok(())
    }

    walk_dir(base_dir, base_dir, &mut html_files)?;
    Ok(html_files)
}

fn get_html_files(path: String) -> Option<Vec<(PathBuf, PathBuf)>> {
    let base_dir = Path::new(&path);

    let Ok(html_files) = find_html_files(base_dir) else {
        return None;
    };
    return Some(html_files);
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterStaticPage {
    signature: String,
    url: String,
    html_content: String,
}
pub fn deploy_static_site(static_path: String) -> Vec<Transaction> {
    let mut transactions: Vec<Transaction> = vec![];
    let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);

    let wasm_data_path = "../apps/static-deployer/out/component.wasm";
    let component_data = std::fs::read(wasm_data_path).expect("need origin site data");

    let create_component_website_tx_data = TransactionData {
        transaction_type: TransactionType::DeployNewComponent,
        calldata: component_data,
    };
    let deploy_website_tx = Transaction {
        pub_key: private_key.public_key(),
        signature: private_key.sign_hash(create_component_website_tx_data.calculate_hash()),
        calldata: create_component_website_tx_data.encode(),
        pow_nonce: rand::thread_rng().r#gen(),
    };

    let site_id = deploy_website_tx.calculate_txhash();

    transactions.push(deploy_website_tx);

    let domain_data = DomainData {
        site_id: site_id,
        domain_name: "vastrum-docs".to_string(),
    };
    let register_domain_tx_data = TransactionData {
        transaction_type: TransactionType::RegisterDomain,
        calldata: domain_data.encode(),
    };
    let register_domain_tx = Transaction {
        pub_key: private_key.public_key(),
        signature: private_key.sign_hash(register_domain_tx_data.calculate_hash()),
        calldata: register_domain_tx_data.encode(),
        pow_nonce: rand::thread_rng().r#gen(),
    };
    transactions.push(register_domain_tx);

    let pages = get_html_files(static_path).unwrap();
    for page in pages {
        let page_route = page.0;
        let page_file_path = page.1;
        println!("adding page route {}", page_route.display());
        println!("adding page file path {}", page_file_path.display());
        let content = fs::read_to_string(&page_file_path).unwrap();

        let add_page = RegisterStaticPage {
            signature: "add_page".to_string(),
            url: page_route.display().to_string(),
            html_content: content,
        };
        let add_page_json = serde_json::to_string(&add_page).unwrap();

        let add_page_tx_data = TransactionData {
            transaction_type: TransactionType::Call,
            calldata: (SiteCall {
                site_id: site_id,
                args: add_page_json.as_bytes().to_vec(),
            })
            .encode()
            .to_vec(),
        };
        let add_page_tx = Transaction {
            pub_key: private_key.public_key(),
            signature: private_key.sign_hash(add_page_tx_data.calculate_hash()),
            calldata: add_page_tx_data.encode(),
            pow_nonce: rand::thread_rng().r#gen(),
        };
        transactions.push(add_page_tx);
    }
    return transactions;
}
#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_get_html() {
        match env::current_dir() {
            Ok(path) => println!("Current directory: {}", path.display()),
            Err(e) => eprintln!("Error getting current directory: {}", e),
        }
        let pages = get_html_files("../apps/static-vastrum-docs".to_string()).unwrap();
        for page in pages {
            println!("page relative route {}", page.0.display());
            println!("page file path {}", page.1.display());
        }
    }
}

use rand::Rng;
use serde::{Deserialize, Serialize};
use shared_types::borsh::BorshExt;
use shared_types::crypto::ed25519;
use shared_types::types::application::domaindata::DomainData;
use shared_types::types::application::sitecall::SiteCall;
use shared_types::types::application::transactiondata::{TransactionData, TransactionType};
use shared_types::types::execution::transaction::Transaction;
use std::fs;
use std::path::{Path, PathBuf};
