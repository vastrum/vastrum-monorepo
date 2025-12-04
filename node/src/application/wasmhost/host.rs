pub struct VastrumHost {
    page_database: Arc<PageDatabase>,
}
impl VastrumHost {
    pub fn execute_component(
        &self,
        component_data: Vec<u8>,
        calldata: Vec<u8>,
        site_id: Sha256Digest,
    ) -> Result<()> {
        let config = common_config();
        let engine = Engine::new(&config)?;
        let component;
        unsafe {
            component = Component::deserialize(&engine, component_data)?;
        }

        //TODO maybe limits memory usage
        //https://docs.rs/wasmtime/latest/wasmtime/struct.Store.html#method.limiter
        //https://docs.wasmtime.dev/examples-deterministic-wasm-execution.html
        //might be much more complicated
        // https://github.com/paritytech/wasm-instrument/blob/master/README.md

        let mut store = Store::new(
            &engine,
            HostState::new(
                site_id,
                self.page_database.clone(),
                StoreLimitsBuilder::new()
                    .memory_size(256 * 1024 * 1024) //256mb
                    .instances(10)
                    .build(),
            ),
        );
        store.limiter(|state| &mut state.limits);

        let mut linker = Linker::new(&engine);
        runtimebindings::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

        let smart_contract = Hostbindings::instantiate(&mut store, &component, &linker)?;
        let calldata = String::from_utf8_lossy(&calldata);

        let _res = smart_contract.contractbindings().call_makecall(&mut store, &calldata);

        Ok(())
    }

    pub fn compile_component(wasm_data: &Vec<u8>) -> Result<Component> {
        let config = common_config();
        let engine = wasmtime::Engine::new(&config).expect("valid config");
        let component = wasmtime::component::Component::from_binary(&engine, wasm_data);
        return component;
    }
    pub fn new() -> VastrumHost {
        return VastrumHost { page_database: Arc::new(PageDatabase::new()) };
    }
}

use crate::{
    application::wasmhost::{
        config::common_config,
        hostbindings::{HostState, Hostbindings, runtimebindings},
    },
    db::pagedb::PageDatabase,
};
use shared_types::crypto::sha256::Sha256Digest;
use std::sync::Arc;
use wasmtime::{
    Engine, Result, Store, StoreLimitsBuilder,
    component::{Component, HasSelf, Linker},
};
