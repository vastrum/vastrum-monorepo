pub struct VastrumHost {
    engine: Engine,
    linker: Linker<HostState>,
}

impl VastrumHost {
    fn make_store(
        &self,
        site_id: Sha256Digest,
        message_sender: ed25519::PublicKey,
        block_timestamp: u64,
        db: Arc<BatchDb>,
    ) -> Store<HostState> {
        let mut store = Store::new(
            &self.engine,
            HostState::new(
                site_id,
                message_sender,
                block_timestamp,
                StoreLimitsBuilder::new()
                    .memory_size(vastrum_shared_types::limits::MAX_WASM_MEMORY)
                    .instances(10)
                    .build(),
                db,
            ),
        );
        store.limiter(|state| &mut state.limits);
        store
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn execute_call(
        &self,
        module: &Module,
        calldata: Vec<u8>,
        site_id: Sha256Digest,
        message_sender: ed25519::PublicKey,
        block_timestamp: u64,
        db: Arc<BatchDb>,
    ) -> Result<()> {
        let mut store = self.make_store(site_id, message_sender, block_timestamp, db);
        vastrum_bindings_host::call_contract(&self.linker, &mut store, module, &calldata)?;
        Ok(())
    }

    pub fn execute_construct(
        &self,
        module_file_path: &std::path::Path,
        constructor_params: Vec<u8>,
        site_id: Sha256Digest,
        message_sender: ed25519::PublicKey,
        block_timestamp: u64,
        db: Arc<BatchDb>,
    ) -> Result<()> {
        let module = unsafe { Module::deserialize_file(&self.engine, module_file_path)? };
        let mut store = self.make_store(site_id, message_sender, block_timestamp, db);
        vastrum_bindings_host::construct_contract(
            &self.linker,
            &mut store,
            &module,
            &constructor_params,
        )?;
        Ok(())
    }

    pub fn compile_module(&self, wasm_data: &[u8]) -> Result<Vec<u8>> {
        let module = Module::new(&self.engine, wasm_data)?;
        module.serialize()
    }

    pub fn new() -> VastrumHost {
        let config = common_config();
        let engine = Engine::new(&config).unwrap();
        let mut linker = Linker::new(&engine);
        vastrum_bindings_host::add_to_linker(&mut linker).unwrap();
        VastrumHost { engine, linker }
    }
}
use super::{config::common_config, hostbindings::HostState};
use crate::db::BatchDb;
use vastrum_shared_types::crypto::{ed25519, sha256::Sha256Digest};
use std::sync::Arc;
use wasmtime::{Engine, Linker, Module, Result, Store, StoreLimitsBuilder};
