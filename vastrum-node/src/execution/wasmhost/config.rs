use vastrum_shared_types::limits::MAX_WASM_MEMORY;

pub fn common_config() -> wasmtime::Config {
    let mut config = wasmtime::Config::new();

    //https://docs.wasmtime.dev/api/wasmtime/struct.Config.html
    //https://github.com/paritytech/substrate/blob/master/client/executor/wasmtime/src/runtime.rs
    //more recent https://github.com/paritytech/polkadot-sdk/blob/master/substrate/client/executor/wasmtime/src/runtime.rs

    config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);

    //https://docs.wasmtime.dev/examples-deterministic-wasm-execution.html
    config.cranelift_nan_canonicalization(true);

    // Disable unused WASM features (reduces code size + JIT validation overhead)
    // Note: reference_types and multi_value are required by Rust-compiled WASM
    config.relaxed_simd_deterministic(true);
    config.wasm_simd(false);
    config.wasm_relaxed_simd(false);
    config.wasm_multi_memory(false);
    config.wasm_threads(false);
    config.wasm_memory64(false);
    config.wasm_tail_call(false);

    config.max_wasm_stack(32 * 1024 * 1024); //32mb
    config.async_stack_size(64 * 1024 * 1024); //64mb, must exceed max_wasm_stack

    config.parallel_compilation(true);

    config.memory_init_cow(true);
    config.memory_guaranteed_dense_image_size(MAX_WASM_MEMORY as u64);

    let mut pool = wasmtime::PoolingAllocationConfig::default();
    pool.max_unused_warm_slots(4);
    pool.max_core_instance_size(512 * 1024); // 512KB per instance
    pool.table_elements(8192);
    pool.total_memories(100);
    pool.total_tables(100);
    pool.max_memory_size(MAX_WASM_MEMORY);
    pool.total_core_instances(100);
    config.allocation_strategy(wasmtime::InstanceAllocationStrategy::Pooling(pool));

    return config;
}
