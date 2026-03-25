use crate::helpers::*;
use crate::{RUNS, TXS_PER_BATCH};

pub fn run() {
    let ctx = &mut BenchContext::new();
    let sel_kvmap_u64_set = selector("kvmap_u64_set");

    run_benchmark("kvmap_u64_set", RUNS, || {
        let mut calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for i in 0..TXS_PER_BATCH {
            calldatas.push(build_calldata(
                sel_kvmap_u64_set,
                &borsh::to_vec(&(format!("user_{i}"), i as u64)).unwrap(),
            ));
        }
        let txs = build_txs(ctx.site_id, calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });
}
