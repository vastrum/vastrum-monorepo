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

    pub fn run_call(
        &self,
        module: &Module,
        calldata: Vec<u8>,
        site_id: Sha256Digest,
        message_sender: ed25519::PublicKey,
        block_timestamp: u64,
        db: Arc<BatchDb>,
    ) -> CallOutcome {
        let mut store = self.make_store(site_id, message_sender, block_timestamp, db);
        if let Err(e) = store.set_fuel(TX_FUEL_CAP) {
            return CallOutcome { fuel: 0, execution_result: Err(e) };
        }
        let execution_result = call_contract(&self.linker, &mut store, module, &calldata);
        let fuel = TX_FUEL_CAP.saturating_sub(store.get_fuel().unwrap_or(0));
        return CallOutcome { fuel, execution_result };
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

        //Todo proper gas limits for constructor calls
        store.set_fuel(BLOCK_FUEL_LIMIT)?;
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

pub struct CallOutcome {
    pub fuel: u64,
    pub execution_result: Result<()>,
}

use super::{config::common_config, hostbindings::HostState};
use crate::db::BatchDb;
use std::sync::Arc;
use vastrum_bindings_host::call_contract;
use vastrum_shared_types::crypto::{ed25519, sha256::Sha256Digest};
use vastrum_shared_types::limits::{BLOCK_FUEL_LIMIT, TX_FUEL_CAP};
use wasmtime::{Engine, Linker, Module, Result, Store, StoreLimitsBuilder};
