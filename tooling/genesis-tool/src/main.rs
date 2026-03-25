#[derive(Parser)]
#[command(about = "Generate validator keystores and genesis config")]
struct Args {
    #[arg(long)]
    validators: u64,
    #[arg(long, default_value = "0")]
    rpc_nodes: u64,
    #[arg(long, default_value = "./genesis")]
    output_dir: PathBuf,
    #[arg(long, value_delimiter = ',')]
    bootstrap_hosts: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    rpc_hosts: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    generate_genesis(
        args.validators,
        args.rpc_nodes,
        args.output_dir,
        args.bootstrap_hosts,
        args.rpc_hosts,
    )
}

fn generate_genesis(
    validators: u64,
    rpc_nodes: u64,
    output_dir: PathBuf,
    bootstrap_hosts: Vec<String>,
    rpc_hosts: Vec<String>,
) -> Result<()> {
    std::fs::create_dir_all(&output_dir)?;

    let mut keystores = Vec::new();
    let mut genesis_validators = Vec::new();

    for i in 0..validators {
        let keystore = Keystore::generate();
        let dir = output_dir.join(format!("validator-{i}"));
        std::fs::create_dir_all(&dir)?;
        keystore.save_to_file(&dir.join("keystore.bin"));

        genesis_validators.push(GenesisValidator {
            validator_index: i,
            validator_pub_key: keystore.validator_private_key.public_key().to_string(),
            p2p_pub_key: keystore.p2p_key.public_key().to_string(),
            stake: 100,
        });

        keystores.push(keystore);
    }

    let mut genesis_bootstrap_peers = Vec::new();
    for (i, host) in bootstrap_hosts.iter().enumerate() {
        genesis_bootstrap_peers.push(GenesisBootstrapPeer {
            p2p_pub_key: keystores[i].p2p_key.public_key().to_string(),
            host: host.clone(),
        });
    }

    let mut genesis_rpc_nodes = Vec::new();
    for i in 0..rpc_nodes as usize {
        let host = rpc_hosts.get(i).map(|h| h.as_str()).unwrap_or("127.0.0.1");
        genesis_rpc_nodes.push(GenesisRpcNode {
            host: host.to_string(),
            fingerprint: keystores[i].dtls_key.fingerprint().to_string(),
        });
    }

    let config = GenesisConfig {
        validators: genesis_validators,
        bootstrap_peers: genesis_bootstrap_peers,
        rpc_nodes: genesis_rpc_nodes,
    };

    let json = serde_json::to_string_pretty(&config)?;
    let genesis_path = output_dir.join("genesis.json");
    std::fs::write(&genesis_path, &json)?;

    println!("Generated {validators} validator keystores");
    println!("Genesis config: {}", genesis_path.display());
    for i in 0..validators {
        println!(
            "  Validator {i}: {}",
            output_dir.join(format!("validator-{i}/keystore.bin")).display()
        );
    }
    let shared_types_genesis = Path::new("shared-types/genesis.json");
    if shared_types_genesis.parent().map(|p| p.exists()).unwrap_or(false) {
        std::fs::copy(&genesis_path, shared_types_genesis)?;
        println!();
        println!("Installed to {}", shared_types_genesis.display());
        println!("Rebuild to apply");
    }

    return Ok(());
}

use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};
use vastrum_node::keystore::keyset::Keystore;
use vastrum_shared_types::genesis::{
    GenesisBootstrapPeer, GenesisConfig, GenesisRpcNode, GenesisValidator,
};
