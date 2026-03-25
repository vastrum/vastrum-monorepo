use crate::helpers::*;
use crate::{RUNS, TXS_PER_BATCH};
use vastrum_shared_types::crypto::{ed25519::PrivateKey, sha256::Sha256Digest};
use vastrum_shared_types::transactioning::transaction_generator::{
    build_call_transaction, build_deploy_new_module_transaction,
};
use std::io::Write;

pub fn run() {
    let ctx = &mut BenchContext::new();
    let sel_add_to_counter = selector("add_to_counter");
    let site_ids = deploy_unique_contracts(ctx, 1000);

    //deploy 1000 contracts to test non cached transactions time
    run_benchmark("add_to_counter_1000_random_non_cached_contracts", RUNS, || {
        let mut txs = Vec::with_capacity(TXS_PER_BATCH);
        for i in 0..TXS_PER_BATCH {
            let calldata = build_calldata(sel_add_to_counter, &borsh::to_vec(&1u32).unwrap());
            txs.push(build_call_transaction(
                site_ids[i % site_ids.len()],
                calldata,
                i as u64,
                PrivateKey::from_seed(i as u64),
                ctx.height,
            ));
        }
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });

    run_benchmark("add_to_counter", RUNS, || {
        let calldata = build_calldata(sel_add_to_counter, &borsh::to_vec(&1u32).unwrap());
        let txs = build_txs(ctx.site_id, vec![calldata; TXS_PER_BATCH], ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });
}

fn deploy_unique_contracts(ctx: &mut BenchContext, count: usize) -> Vec<Sha256Digest> {
    let wasm_bytes = std::fs::read("contract/out/contract.wasm").expect("failed to read wasm");
    let mut site_ids = Vec::with_capacity(count);
    let batch_size = 100;
    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(count);
        let mut deploy_txs = Vec::with_capacity(batch_end - batch_start);
        for i in batch_start..batch_end {
            let unique_wasm = make_unique_wasm(&wasm_bytes, i);
            let tx = build_deploy_new_module_transaction(
                unique_wasm,
                vec![],
                i as u64,
                PrivateKey::from_seed(i as u64),
                ctx.height + 1,
            );
            site_ids.push(tx.calculate_txhash());
            deploy_txs.push(tx);
        }
        execute_block(&mut ctx.execution, &mut ctx.height, deploy_txs);
        print!("\rdeploying contracts: {}/{}", batch_end, count);
        std::io::stdout().flush().unwrap();
    }
    println!();
    println!("settling after deploy...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    site_ids
}
