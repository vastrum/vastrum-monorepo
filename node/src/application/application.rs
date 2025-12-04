pub struct Application {
    vastrum_host: Arc<VastrumHost>,
    wasm_database: Arc<ComponentDatabase>,
    site_database: Arc<SiteDatabase>,
    domain_database: Arc<DomainDatabase>,
}
impl Application {
    pub fn new() -> Application {
        return Application {
            vastrum_host: Arc::new(VastrumHost::new()),
            wasm_database: Arc::new(ComponentDatabase::new()),
            site_database: Arc::new(SiteDatabase::new()),
            domain_database: Arc::new(DomainDatabase::new()),
        };
    }
    pub fn execute_transaction(&self, calldata: Vec<u8>, tx_hash: Sha256Digest) {
        let transaction_data = borsh::from_slice::<TransactionData>(&calldata).unwrap();

        info!("executing transaction {transaction_data:#?}");
        let calldata = transaction_data.calldata;

        if transaction_data.transaction_type == TransactionType::Call {
            self.execute_call_tx(calldata);
        } else if transaction_data.transaction_type == TransactionType::DeployNewComponent {
            self.execute_deploy_new_component_tx(calldata, tx_hash);
        } else if transaction_data.transaction_type == TransactionType::AddComponent {
            // self.execute_add_component_transaction(transaction_call_data);
        } else if transaction_data.transaction_type == TransactionType::DeployStoredComponent {
        } else if transaction_data.transaction_type == TransactionType::RegisterDomain {
            self.register_domain(calldata);
        }
    }

    pub fn execute_call_tx(&self, calldata: Vec<u8>) {
        let site_call = borsh::from_slice::<SiteCall>(&calldata).unwrap();
        let site_id = site_call.site_id;

        let Some(site_data) = self.site_database.read_site_data(site_id) else {
            panic!("could not find site data");
            //return;
        };

        let Some(component_data) = self.wasm_database.read_component(site_data.component_id) else {
            panic!("could not find component data");
            //return;
        };

        self.vastrum_host.execute_component(component_data, site_call.args, site_id).unwrap();
    }

    fn register_domain(&self, calldata: Vec<u8>) {
        let domain_data = DomainData::decode(&calldata).expect("todo");
        println!("deploying domain {domain_data:#?}");
        let current_domain = self.domain_database.read_site_data(&domain_data.domain_name);
        println!("current domain {current_domain:#?}");
        if current_domain.is_none() {
            println!("writing domain data");
            self.domain_database.write(domain_data);
        }
    }
    pub fn execute_deploy_new_component_tx(&self, calldata: Vec<u8>, tx_hash: Sha256Digest) {
        let wasm_component_key = tx_hash;
        self.add_new_component_to_wasm_store(&calldata, wasm_component_key);
        let site_id_key = tx_hash;
        self.create_new_site(site_id_key, wasm_component_key);
        println!("added new component {wasm_component_key:#?} and deployed at {site_id_key:#?}");

        //always call deploy() on all newly deployed components
        let deploy_contract_json = r#"{"signature":"deploy"}"#;
        let deploy_tx_data = TransactionData {
            transaction_type: TransactionType::Call,
            calldata: (SiteCall {
                site_id: site_id_key,
                args: deploy_contract_json.as_bytes().to_vec(),
            })
            .encode(),
        };
        self.execute_call_tx(deploy_tx_data.calldata);
    }

    fn add_new_component_to_wasm_store(&self, wasm_data: &Vec<u8>, key: Sha256Digest) {
        //check if component already compiled
        if let Some(_component) = self.wasm_database.read_component(key) {
            return;
        }

        let component = VastrumHost::compile_component(wasm_data);
        if let Ok(component) = component {
            let serialized_component = component.serialize().expect("serialize should not fail");

            let wasm_component = WasmComponent { key: key, data: serialized_component };
            self.wasm_database.write(wasm_component);
        }
    }

    fn create_new_site(&self, site_id_key: Sha256Digest, wasm_component_key: Sha256Digest) {
        let site_data = SiteData { site_id: site_id_key, component_id: wasm_component_key };
        self.site_database.write(site_data);
    }
}

use shared_types::{
    borsh::BorshExt,
    crypto::sha256::Sha256Digest,
    types::application::{
        domaindata::DomainData,
        sitecall::SiteCall,
        transactiondata::{TransactionData, TransactionType},
    },
};
use std::sync::Arc;
use tracing::info;

use crate::{
    application::{
        sitedata::SiteData, wasm_component_data::WasmComponent, wasmhost::host::VastrumHost,
    },
    db::{componentdb::ComponentDatabase, domaindb::DomainDatabase, sitedb::SiteDatabase},
};
