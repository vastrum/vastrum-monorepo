use crate::TXS_PER_BATCH;

pub struct BenchContext {
    pub execution: Execution,
    pub height: u64,
    pub site_id: Sha256Digest,
}

impl BenchContext {
    pub fn new() -> Self {
        build_contract("contract", "contract/out");

        let db = Arc::new(Db::open_fresh("/tmp/vastrum-benchmark-db"));
        let mut execution = Execution::new(db);
        let mut height: u64 = 0;

        // Deploy contract
        let wasm_bytes = std::fs::read("contract/out/contract.wasm").expect("failed to read wasm");
        let deploy_tx =
            build_deploy_new_module_transaction(wasm_bytes, vec![], 0, PrivateKey::from_seed(0), 0);
        let site_id = deploy_tx.calculate_txhash();
        execute_block(&mut execution, &mut height, vec![deploy_tx]);

        BenchContext { execution, height, site_id }
    }
}

pub fn selector(name: &str) -> [u8; 8] {
    calculate_function_selector(name)
}

pub fn build_calldata(selector: [u8; 8], args: &[u8]) -> Vec<u8> {
    let mut calldata = Vec::with_capacity(8 + args.len());
    calldata.extend_from_slice(&selector);
    calldata.extend_from_slice(args);
    calldata
}

pub fn build_txs(
    site_id: Sha256Digest,
    calldata_list: Vec<Vec<u8>>,
    height: u64,
) -> Vec<Transaction> {
    let mut txs = Vec::with_capacity(calldata_list.len());
    for (i, calldata) in calldata_list.into_iter().enumerate() {
        txs.push(build_call_transaction(
            site_id,
            calldata,
            i as u64,
            PrivateKey::from_seed(i as u64),
            height,
        ));
    }
    txs
}

pub fn execute_block(
    execution: &mut Execution,
    height: &mut u64,
    txs: Vec<Transaction>,
) -> Duration {
    *height += 1;
    let block = Block {
        height: *height,
        transactions: txs,
        previous_block_hash: Sha256Digest::from([0u8; 32]),
        timestamp: 0,
        previous_block_state_root: Sha256Digest::default(),
    };
    let finalized = FinalizedBlock { block, votes: BTreeMap::new(), round: 0 };
    let start = Instant::now();
    execution.execute_block(finalized);
    start.elapsed()
}

pub fn seed_btree(ctx: &mut BenchContext, target: usize, sel_kvbtree_u64_insert: [u8; 8]) {
    const BATCH_SIZE: usize = 1000;
    for batch_start in (0..target).step_by(BATCH_SIZE) {
        let batch_end = (batch_start + BATCH_SIZE).min(target);
        let mut calldatas = Vec::with_capacity(batch_end - batch_start);
        for _ in batch_start..batch_end {
            let key = rand::random::<u64>() % 100_000_000;
            calldatas
                .push(build_calldata(sel_kvbtree_u64_insert, &borsh::to_vec(&(key, key)).unwrap()));
        }
        let txs = build_txs(ctx.site_id, calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs);
        print!("\rseeding btree: {}/{}", batch_end, target);
        std::io::stdout().flush().unwrap();
    }
    println!();
}

pub fn make_unique_wasm(base: &[u8], i: usize) -> Vec<u8> {
    let mut wasm = base.to_vec();
    let payload = i.to_le_bytes();
    let name = b"id";
    let section_len = 1 + name.len() + payload.len();
    wasm.push(0); // custom section id
    leb128_encode(&mut wasm, section_len);
    leb128_encode(&mut wasm, name.len());
    wasm.extend_from_slice(name);
    wasm.extend_from_slice(&payload);
    wasm
}

fn leb128_encode(buf: &mut Vec<u8>, mut value: usize) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

pub fn run_benchmark<F>(name: &str, runs: usize, mut bench_fn: F)
where
    F: FnMut() -> Duration,
{
    let mut durations = Vec::with_capacity(runs);

    for run in 0..runs {
        let elapsed = bench_fn();
        durations.push(elapsed);
        print!("\r{}: run {}/{}", name, run + 1, runs);
        std::io::stdout().flush().unwrap();
    }
    println!();

    let total: Duration = durations.iter().sum();
    let avg_duration = total / runs as u32;
    let avg_tps = (TXS_PER_BATCH as f64 * runs as f64) / total.as_secs_f64();
    let avg_per_tx_ms = avg_duration.as_secs_f64() * 1000.0 / TXS_PER_BATCH as f64;

    println!("{}:", name);
    println!("{} runs, {} txs per run, total {}", runs, TXS_PER_BATCH, runs * TXS_PER_BATCH);
    println!("avg per tx: {:.4}ms  avg tps: {:.2}", avg_per_tx_ms, avg_tps);
    println!();
}

use vastrum_native_lib::deployers::build::build_contract;
use vastrum_runtime_shared::calculate_function_selector;
use vastrum_shared_types::{
    crypto::{ed25519::PrivateKey, sha256::Sha256Digest},
    transactioning::transaction_generator::{
        build_call_transaction, build_deploy_new_module_transaction,
    },
    types::execution::transaction::Transaction,
};
use std::{
    collections::BTreeMap,
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};
use vastrum_node::{
    consensus::types::{Block, FinalizedBlock},
    db::Db,
    execution::execution::Execution,
};
