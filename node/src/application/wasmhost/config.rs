pub fn common_config() -> wasmtime::Config {
    let mut config = wasmtime::Config::new();

    config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);

    //https://docs.wasmtime.dev/examples-deterministic-wasm-execution.html
    config.cranelift_nan_canonicalization(true);

    //disable simd
    config.relaxed_simd_deterministic(true);
    config.wasm_simd(false);
    config.wasm_relaxed_simd(false);

    //https://github.com/paritytech/substrate/blob/master/client/executor/wasmtime/src/runtime.rs
    //more recent https://github.com/paritytech/polkadot-sdk/blob/master/substrate/client/executor/wasmtime/src/runtime.rs
    config.max_wasm_stack(32 * 1024 * 1024);

    config.parallel_compilation(true);

    config.memory_init_cow(false);

    return config;
}
