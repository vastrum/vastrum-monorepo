#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        name: String,
        #[arg(long, default_value = "site")]
        template: String,
    },
    RunDev {},
    StartNode {
        #[arg(long)]
        keystore: Option<PathBuf>,
        #[arg(long)]
        rpc: bool,
    },
    GenerateKeys {
        #[arg(long, default_value = "keystore.bin")]
        output: PathBuf,
        #[arg(long)]
        wallet_key: String,
    },
    ShowKeys {
        #[arg(long, default_value = "keystore.bin")]
        keystore: PathBuf,
    },
    VastrumGitClone {
        repo_name: String,
    },
    VastrumGitPush {
        repo_name: String,
        private_key: String,
    },
    StartGitterHttpRelay {
        #[arg(long, default_value = "relay.key")]
        relay_key: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { name, template } => {
            scaffold::initialize_new_project(name, template);
        }
        Commands::RunDev {} => start_run_dev().await,
        Commands::StartNode { keystore, rpc } => node::start_node(keystore, rpc).await,
        Commands::GenerateKeys { output, wallet_key } => node::generate_keys(output, wallet_key)?,
        Commands::ShowKeys { keystore } => node::show_keys(keystore),
        Commands::VastrumGitClone { repo_name } => vastrum_git_clone(repo_name).await?,
        Commands::VastrumGitPush { repo_name, private_key } => {
            vastrum_git_push(repo_name, private_key).await?
        }
        Commands::StartGitterHttpRelay { relay_key } => vastrum_git_relay::run(relay_key).await?,
    }
    Ok(())
}

pub mod localnet;
pub mod node;
pub mod scaffold;
pub mod vastrum_git;

use crate::{
    localnet::run_localnet::start_run_dev,
    vastrum_git::{vastrum_git_clone, vastrum_git_push},
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
