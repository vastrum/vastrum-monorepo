pub async fn vastrum_git_clone(repo_name: String) -> Result<()> {
    let site_id = resolve_gitter_site_id().await?;

    let client = ContractAbiClient::new(site_id);
    let pb = new_counter_bar("Cloning");
    clone_repo(&repo_name, &repo_name, &client, Some(&pb)).await?;
    pb.finish_and_clear();
    println!("Clone complete.");
    Ok(())
}

pub async fn vastrum_git_push(repo_name: String, private_key: String) -> Result<()> {
    let site_id = resolve_gitter_site_id().await?;
    let account_private_key = ed25519::PrivateKey::try_from_string(private_key)
        .ok_or_else(|| anyhow!("invalid private key"))?;

    let site_private_key = derive_site_key(&account_private_key, site_id);
    let pubkey = site_private_key.public_key();
    let client = ContractAbiClient::new(site_id).with_account_key(site_private_key);

    let is_owner = publickey_is_owner_of_repo(&repo_name, pubkey, &client).await?;
    if !is_owner {
        anyhow::bail!("push rejected: you are not the owner of this repository");
    }

    let pb = new_progress_bar("Checking");
    let push_result = push_to_repo(".", &repo_name, &client, Some(&pb)).await?;
    match push_result {
        PushOutcome::Pushed { objects_uploaded } => {
            pb.finish_and_clear();
            println!("Push complete. {} objects uploaded.", objects_uploaded);
        }
        PushOutcome::AlreadyUpToDate => {
            pb.finish_and_clear();
            println!("Already up to date.");
        }
    }
    Ok(())
}

async fn resolve_gitter_site_id() -> Result<Sha256Digest> {
    let http = NativeHttpClient::new();
    let site_id = http
        .resolve_domain(GITTER_DOMAIN.to_string())
        .await?
        .ok_or_else(|| anyhow!("could not resolve domain: {}", GITTER_DOMAIN))?;
    Ok(site_id)
}

fn new_counter_bar(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}: {pos} objects [{elapsed_precise}]")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    return pb;
}

fn new_progress_bar(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {msg} [{bar:30.cyan/blue}] {pos}/{len} [{elapsed_precise}]",
        )
        .unwrap()
        .progress_chars("##-")
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    return pb;
}

use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use vastrum_git_lib::{
    ContractAbiClient,
    config::GITTER_DOMAIN,
    native::{
        clone::clone_repo,
        upload::{PushOutcome, push_to_repo},
    },
    universal::utils::publickey_is_owner_of_repo,
};
use vastrum_native_lib::NativeHttpClient;
use vastrum_shared_types::crypto::{ed25519, sha256::Sha256Digest, site_key::derive_site_key};
