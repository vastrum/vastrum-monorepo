use crate::helpers::*;
use crate::{RUNS, TXS_PER_BATCH};

pub fn run() {
    let ctx = &mut BenchContext::new();
    let sel_insert = selector("kvbtree_u64_insert");
    let sel_remove = selector("kvbtree_u64_remove");
    let sel_get = selector("kvbtree_u64_get");

    run_benchmark("kvbtree_u64_insert", RUNS, || {
        let mut calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for _ in 0..TXS_PER_BATCH {
            let key = rand::random::<u64>() % 100_000_000;
            calldatas.push(build_calldata(sel_insert, &borsh::to_vec(&(key, key)).unwrap()));
        }
        let txs = build_txs(ctx.site_id, calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });

    // Seed to 10k
    seed_btree(ctx, 10_000, sel_insert);
    run_benchmark("kvbtree_u64_insert_with_10k_existing_entries", RUNS, || {
        let mut calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for _ in 0..TXS_PER_BATCH {
            let key = rand::random::<u64>() % 100_000_000;
            calldatas.push(build_calldata(sel_insert, &borsh::to_vec(&(key, key)).unwrap()));
        }
        let txs = build_txs(ctx.site_id, calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });

    run_benchmark("kvbtree_u64_get_with_10k_existing_entries", RUNS, || {
        let mut calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for _ in 0..TXS_PER_BATCH {
            let key = rand::random::<u64>() % 100_000_000;
            calldatas.push(build_calldata(sel_get, &borsh::to_vec(&key).unwrap()));
        }
        let txs = build_txs(ctx.site_id, calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });

    // Seed to 100k
    seed_btree(ctx, 100_000, sel_insert);
    run_benchmark("kvbtree_u64_insert_with_100k_existing_entries", RUNS, || {
        let mut calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for _ in 0..TXS_PER_BATCH {
            let key = rand::random::<u64>() % 100_000_000;
            calldatas.push(build_calldata(sel_insert, &borsh::to_vec(&(key, key)).unwrap()));
        }
        let txs = build_txs(ctx.site_id, calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });

    run_benchmark("kvbtree_u64_get_with_100k_existing_entries", RUNS, || {
        let mut calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for _ in 0..TXS_PER_BATCH {
            let key = rand::random::<u64>() % 100_000_000;
            calldatas.push(build_calldata(sel_get, &borsh::to_vec(&key).unwrap()));
        }
        let txs = build_txs(ctx.site_id, calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });

    // Remove with 100k+ entries
    run_benchmark("kvbtree_u64_remove_with_100k_existing_entries", RUNS, || {
        // Insert known keys (untimed)
        let mut keys = Vec::with_capacity(TXS_PER_BATCH);
        for _ in 0..TXS_PER_BATCH {
            keys.push(rand::random::<u64>() % 100_000_000);
        }
        let mut insert_calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for &k in &keys {
            insert_calldatas.push(build_calldata(sel_insert, &borsh::to_vec(&(k, k)).unwrap()));
        }
        let insert_txs = build_txs(ctx.site_id, insert_calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, insert_txs);

        // Time removal
        let mut remove_calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for &k in &keys {
            remove_calldatas.push(build_calldata(sel_remove, &borsh::to_vec(&k).unwrap()));
        }
        let remove_txs = build_txs(ctx.site_id, remove_calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, remove_txs)
    });

    // Seed to 1M
    seed_btree(ctx, 1_000_000, sel_insert);
    run_benchmark("kvbtree_u64_insert_with_1m_existing_entries", RUNS, || {
        let mut calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for _ in 0..TXS_PER_BATCH {
            let key = rand::random::<u64>() % 100_000_000;
            calldatas.push(build_calldata(sel_insert, &borsh::to_vec(&(key, key)).unwrap()));
        }
        let txs = build_txs(ctx.site_id, calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });

    run_benchmark("kvbtree_u64_get_with_1m_existing_entries", RUNS, || {
        let mut calldatas = Vec::with_capacity(TXS_PER_BATCH);
        for _ in 0..TXS_PER_BATCH {
            let key = rand::random::<u64>() % 100_000_000;
            calldatas.push(build_calldata(sel_get, &borsh::to_vec(&key).unwrap()));
        }
        let txs = build_txs(ctx.site_id, calldatas, ctx.height);
        execute_block(&mut ctx.execution, &mut ctx.height, txs)
    });
}
