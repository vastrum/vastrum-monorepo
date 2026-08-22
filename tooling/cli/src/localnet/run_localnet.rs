pub async fn start_run_dev() {
    //use local genesis-dev.json instead of production genesis.json
    unsafe { std::env::set_var("VASTRUM_LOCALNET", "1") };
    if !valid_directory() {
        println!(
            "Not a valid directory to start dev server in, no deploy crate found, need to have a deploy subdirectory of the directory you execute vastrum-cli run-dev in"
        );
        return;
    }

    let mut node = tokio::spawn(start_localnet(false));

    tokio::select! {
        _ = wait_for_rpc_server() => {}
        result = &mut node => node_died(result),
    }

    start_browser();

    let mut deploy = tokio::process::Command::new("cargo")
        .args(["run"])
        .current_dir("deploy")
        .spawn()
        .expect("failed to spawn deploy");

    tokio::select! {
        status = deploy.wait() => {
            let status = status.expect("deploy process error");
            if !status.success() {
                eprintln!("deploy failed with {status}");
                std::process::exit(1);
            }
        }
        result = &mut node => {
            let _ = deploy.kill().await;
            node_died(result);
        }
    }

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        result = &mut node => node_died(result),
    }
}

fn node_died(result: Result<(), tokio::task::JoinError>) -> ! {
    match result {
        Ok(()) => eprintln!("node exited unexpectedly"),
        Err(e) => eprintln!("node failed: {e}"),
    }
    std::process::exit(1)
}

fn start_browser() {
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let url = format!("http://index.localhost:{HTTP_RPC_PORT}");
        if let Err(e) = open_in_browser(&url) {
            eprintln!("Failed to open browser: {e}");
        }
    });
}

fn open_in_browser(url: &str) -> std::io::Result<()> {
    let mut command = if cfg!(target_os = "macos") {
        let mut c = Command::new("open");
        c.arg(url);
        c
    } else if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", url]);
        c
    } else {
        let mut c = Command::new("xdg-open");
        c.arg(url);
        c
    };
    let status =
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("browser launcher exited with {status}")))
    }
}

fn valid_directory() -> bool {
    Path::new("deploy/Cargo.toml").exists()
}

async fn wait_for_rpc_server() {
    let addr = SocketAddr::from(([127, 0, 0, 1], HTTP_RPC_PORT));
    while tokio::net::TcpStream::connect(addr).await.is_err() {
        sleep(Duration::from_millis(10)).await;
    }
}

use std::{
    net::SocketAddr,
    path::Path,
    process::{Command, Stdio},
};
use tokio::time::{Duration, sleep};
use vastrum_node::start_localnet;
use vastrum_shared_types::ports::HTTP_RPC_PORT;
