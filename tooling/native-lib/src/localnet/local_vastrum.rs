use vastrum_node::start_localnet;

pub async fn start_local_vastrum_network() {
    tokio::spawn(async {
        start_localnet().await;
    });
}
