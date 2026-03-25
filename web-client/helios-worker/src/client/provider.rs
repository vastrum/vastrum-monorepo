#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct HeliosConfig {
    pub execution_rpc: String,
    pub consensus_rpc: String,
    pub checkpoint: String,
    pub network: String,
}

thread_local! {
    static HELIOS_CONFIG: RefCell<Option<HeliosConfig>> = const { RefCell::new(None) };
    pub static HELIOS_VASTRUM: RefCell<GlobalVastrumHeliosProvider> = RefCell::new(GlobalVastrumHeliosProvider::default());
}

#[derive(Default)]
struct GlobalVastrumHeliosProvider {
    pub client: Option<Rc<EthereumClient>>,
}

pub fn set_helios_config(config: HeliosConfig) {
    HELIOS_CONFIG.with(|c| *c.borrow_mut() = Some(config));
}

pub fn reset_eth_client() {
    HELIOS_VASTRUM.with(|state| {
        state.borrow_mut().client = None;
    });
}

pub fn get_eth_client() -> Rc<EthereumClient> {
    HELIOS_VASTRUM.with(|state| {
        if let Some(ref client) = state.borrow().client {
            return Rc::clone(client);
        }

        let config = HELIOS_CONFIG
            .with(|c| c.borrow().clone())
            .expect("HeliosConfig not set - call init_helios() first");

        let client = Rc::new(
            EthereumClient::new(
                config.execution_rpc,
                config.consensus_rpc,
                config.network,
                config.checkpoint,
            )
            .unwrap(),
        );

        state.borrow_mut().client = Some(Rc::clone(&client));
        return client;
    })
}

use super::ethereum_client::EthereumClient;
use std::cell::RefCell;
use std::rc::Rc;
