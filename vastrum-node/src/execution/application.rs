impl Execution {
    pub fn execute_call_tx(&self, calldata: Vec<u8>, module_cache: &HashMap<PathBuf, Module>) {
        let Ok(site_call) = borsh::from_slice::<SiteCall>(&calldata) else {
            tracing::warn!("failed to decode SiteCall");
            return;
        };
        self.call_site(site_call.site_id, site_call.calldata, module_cache);
    }

    fn call_site(
        &self,
        site_id: Sha256Digest,
        calldata: Vec<u8>,
        module_cache: &HashMap<PathBuf, Module>,
    ) {
        let Some(site_data) = self.db.read_site(site_id) else {
            tracing::warn!("site not found: {site_id:?}");
            return;
        };
        let module_file_path = self.db.calculate_module_file_path(site_data.module_id);
        let Some(module) = self.load_module(&module_file_path, module_cache) else {
            return;
        };
        //incase tx fails revert state changes writen to db by this tx
        self.db.begin_revertable();
        let result = self.vastrum_host.execute_call(
            &module,
            calldata,
            site_id,
            self.message_sender,
            self.block_timestamp,
            self.db.clone(),
        );
        if let Err(e) = result {
            self.db.rollback_revertable();
            tracing::warn!("execute_call failed: {e:?}");
        } else {
            self.db.commit_revertable();
        }
    }

    fn load_module(
        &self,
        path: &PathBuf,
        module_cache: &HashMap<PathBuf, Module>,
    ) -> Option<Module> {
        if let Some(module) = module_cache.get(path) {
            return Some(module.clone());
        }
        if !path.exists() {
            tracing::warn!("module file not found: {path:?}");
            return None;
        }
        match unsafe { Module::deserialize_file(self.vastrum_host.engine(), path) } {
            Ok(module) => Some(module),
            Err(e) => {
                tracing::warn!("failed to load module: {e:?}");
                None
            }
        }
    }

    pub fn register_domain(&self, calldata: Vec<u8>) {
        let Ok(domain_data) = DomainData::decode(&calldata) else {
            tracing::warn!("failed to decode DomainData");
            return;
        };
        // Reject domain names that look like a site_id
        if Sha256Digest::from_string(&domain_data.domain_name).is_some() {
            tracing::warn!("domain name rejected: looks like a site_id");
            return;
        }
        println!("deploying domain {domain_data:#?}");
        let current_domain = self.db.read_domain(&domain_data.domain_name);
        println!("current domain {current_domain:#?}");
        if current_domain.is_none() {
            println!("writing domain data");
            self.db.write_domain(domain_data);
        }
    }

    /// Upload new wasm, create a site, and call its constructor
    pub fn execute_deploy_new_module_tx(&self, calldata: Vec<u8>, tx_hash: Sha256Digest) {
        let Ok(deploy) = borsh::from_slice::<DeployNewModuleCall>(&calldata) else {
            tracing::warn!("failed to decode DeployNewModuleCall");
            return;
        };
        let Some(module_id) = self.add_new_module_to_wasm_store(&deploy.wasm_data) else {
            return;
        };
        self.deploy_site(module_id, deploy.constructor_calldata, tx_hash);
    }

    /// Store contract wasm bytecode, but dont deploy a site
    pub fn execute_add_module_tx(&self, calldata: Vec<u8>) {
        let _ = self.add_new_module_to_wasm_store(&calldata);
    }

    /// Create a new site from an already stored wasm module and call its constructor
    pub fn execute_deploy_stored_module_tx(&self, calldata: Vec<u8>, tx_hash: Sha256Digest) {
        let Ok(deploy) = borsh::from_slice::<DeployStoredModuleCall>(&calldata) else {
            tracing::warn!("failed to decode DeployStoredModuleCall");
            return;
        };
        self.deploy_site(deploy.module_id, deploy.constructor_calldata, tx_hash);
    }

    fn deploy_site(
        &self,
        module_id: Sha256Digest,
        constructor_calldata: Vec<u8>,
        site_id: Sha256Digest,
    ) {
        let module_file_path = self.db.calculate_module_file_path(module_id);
        if !module_file_path.exists() {
            tracing::warn!("module file not found: {module_id:?}");
            return;
        }
        //incase tx fails revert state changes writen to db by this tx
        self.db.begin_revertable();

        let site_data = SiteData { site_id, module_id };
        self.db.write_site(site_data);

        let result = self.vastrum_host.execute_construct(
            &module_file_path,
            constructor_calldata,
            site_id,
            self.message_sender,
            self.block_timestamp,
            self.db.clone(),
        );
        if let Err(e) = result {
            self.db.rollback_revertable();
            tracing::warn!("execute_construct failed: {e:?}");
            return;
        }
        self.db.commit_revertable();
    }

    fn add_new_module_to_wasm_store(&self, wasm_data: &[u8]) -> Option<Sha256Digest> {
        if wasm_data.len() > MAX_WASM_MODULE_SIZE {
            tracing::warn!("wasm module too large: {} bytes", wasm_data.len());
            return None;
        }
        let key = sha256_hash(wasm_data);

        //check if module already compiled
        if self.db.calculate_module_file_path(key).exists() {
            return Some(key);
        }

        let serialized_module = match self.vastrum_host.compile_module(wasm_data) {
            Ok(module) => module,
            Err(e) => {
                tracing::warn!("failed to compile wasm module: {e}");
                return None;
            }
        };
        self.db.write_module(CompiledModule { key, data: serialized_module });
        return Some(key);
    }
}

use super::{
    execution::Execution,
    types::{compiled_module::CompiledModule, sitedata::SiteData},
};
use vastrum_shared_types::{
    borsh::BorshExt,
    crypto::sha256::{Sha256Digest, sha256_hash},
    limits::MAX_WASM_MODULE_SIZE,
    types::application::{
        deploy_new_module::DeployNewModuleCall, deploy_stored_module::DeployStoredModuleCall,
        domaindata::DomainData, sitecall::SiteCall,
    },
};
use std::{collections::HashMap, path::PathBuf};
use wasmtime::Module;
