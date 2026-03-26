pub fn ensure_localnet(contract_dir: &str, out_dir: &str) {
    static NETWORK: OnceLock<()> = OnceLock::new();
    NETWORK.get_or_init(|| {
        check_ports_available(&[
            (vastrum_shared_types::ports::HTTP_RPC_PORT, "RPC"),
            (vastrum_shared_types::ports::P2P_PORT, "P2P"),
        ]);
        eprintln!("Building contract WASM...");
        crate::deployers::build::build_contract(contract_dir, out_dir);
        eprintln!("Starting localnet...");
        thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                crate::localnet::local_vastrum::start_local_vastrum_network().await;
                loop {
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                }
            });
        });
        eprintln!("Waiting for RPC server...");
        wait_for_rpc_server();
        eprintln!("Localnet ready.");
    });
}
fn wait_for_rpc_server() {
    let addr = SocketAddr::from(([127, 0, 0, 1], vastrum_shared_types::ports::HTTP_RPC_PORT));
    while TcpStream::connect(addr).is_err() {
        thread::sleep(Duration::from_millis(10));
    }
}

fn check_ports_available(ports: &[(u16, &str)]) {
    for &(port, name) in ports {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if TcpListener::bind(addr).is_err() {
            panic!(
                "\n\nPort {port} ({name}) is already in use.\n\
                 A stale test process may be running.\n\
                 Kill it: kill $(lsof -ti:{port})\n"
            );
        }
    }
}

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
