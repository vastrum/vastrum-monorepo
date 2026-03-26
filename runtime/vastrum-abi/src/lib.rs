//because macro crates cannot reexport modules, have to do another non macro crate to reexported needed dependencies for macro
//otherwise would have to separately import each needed dependency for the generated macro code
//now only need to import this crate to use abi macro

#[doc(hidden)]
pub mod __private {
    pub use borsh;
    pub use vastrum_native_types;
    pub use vastrum_rpc_client;
    pub use vastrum_runtime_shared;
    pub use vastrum_shared_types;

    //for deploying contracts
    #[cfg(not(target_arch = "wasm32"))]
    pub use vastrum_native_lib;
}
