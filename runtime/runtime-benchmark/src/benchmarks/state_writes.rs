use crate::helpers::*;
use crate::{RUNS, TXS_PER_BATCH};
use rand::Rng;

pub fn run() {
    let ctx = &mut BenchContext::new();
    let large_payload: String = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(100 * 1024)
        .map(|b| b as char)
        .collect();

    let sel_state_string_set = selector("state_string_set");
    let sel_kvvec_string_push = selector("kvvec_string_push");
    let sel_kvvecbtree_struct_insert = selector("kvvecbtree_struct_insert");

    run_benchmark("state_string_write(100kb)", RUNS, || {
        let calldata =
            build_calldata(sel_state_string_set, &borsh::to_vec(&large_payload).unwrap());
        let txs = build_txs(ctx.site_id, vec![calldata; TXS_PER_BATCH], ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });

    run_benchmark("kvvec_string_push(100kb)", RUNS, || {
        let calldata =
            build_calldata(sel_kvvec_string_push, &borsh::to_vec(&large_payload).unwrap());
        let txs = build_txs(ctx.site_id, vec![calldata; TXS_PER_BATCH], ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });

    run_benchmark("kvvecbtree_struct_insert(100kb)", RUNS, || {
        let mut calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for i in 0..TXS_PER_BATCH {
            calldatas.push(build_calldata(
                sel_kvvecbtree_struct_insert,
                &borsh::to_vec(&(
                    format!("Post {i}"),
                    large_payload.as_str(),
                    format!("author_{i}"),
                ))
                .unwrap(),
            ));
        }
        let txs = build_txs(ctx.site_id, calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });
}
