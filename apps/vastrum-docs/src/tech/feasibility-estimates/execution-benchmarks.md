# Execution Benchmarks


These execution benchmarks were created using [runtime/runtime-benchmark (gitter preview)](https://gitter.vastrum.net/repo/vastrum/tree/runtime/runtime-benchmark/src/main.rs)


These benchmarks are based on 100% block execution, in a production deployment perhaps 10-30% of the block time could be spent executing the block.

Based on these benchmarks i think the primary constraint for scaling TPS will be consensus + bandwidth efficient block propagation.

The benchmark numbers seems to be unreasonably high so it is possible it misrepresents actual TPS capability. I have not benchmarked with large RocksDB state such as with 1 TB+ so it is possible this makes the TPS go down significantly. 

These benchmarks include full block execution including jellyfish merkle tree state hash updates.


These benchmarks were ran with the transaction indexer for the blockchain explorer disabled. 
These decrease TPS by roughly 75%, and they are probably not needed in production deployment. 


## Benchmark results

### Summary

#### Contract Execution

| Benchmark | Avg per tx (ms) | Avg TPS |
|---|---|---|
| Counter (1000 random contracts) | 0.0755 | 13,247 |
| Counter (1 contract) | 0.0186 | 53,815 |

#### KvBTree Operations

| Benchmark | Avg per tx (ms) | Avg TPS |
|---|---|---|
| Insert (empty tree) | 0.0294 | 34,036 |
| Insert (10K existing entries) | 0.0379 | 26,390 |
| Get (10K existing entries) | 0.0273 | 36,694 |
| Insert (100K existing entries) | 0.0504 | 19,854 |
| Get (100K existing entries) | 0.0312 | 32,025 |
| Remove (100K existing entries) | 0.0524 | 19,085 |
| Insert (1M existing entries) | 0.0876 | 11,420 |
| Get (1M existing entries) | 0.0387 | 25,859 |

#### KvMap Operations

| Benchmark | Avg per tx (ms) | Avg TPS |
|---|---|---|
| KvMap u64 set | 0.0310 | 32,235 |

#### Large Payload (100KB)

| Benchmark | Avg per tx (ms) | Avg TPS |
|---|---|---|
| State string write | 0.3717 | 2,690 |
| KvVec string push | 0.6267 | 1,596 |
| KvVecBTree struct insert | 0.7200 | 1,389 |


## To replicate these numbers

Comment out this in execution.rs
```rust
self.db.set_tx_as_included(tx_hash);
```
And this in execution.rs
```rust
indexer::index_finalized_block(&self.db, &finalized);
```

To run benchmarks, in root
        
        make run_benchmark



### Raw output

        deploying contracts: 1000/1000
        settling after deploy...
        add_to_counter_1000_random_non_cached_contracts: run 10/10
        add_to_counter_1000_random_non_cached_contracts:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0755ms  avg tps: 13246.69

        add_to_counter: run 10/10
        add_to_counter:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0186ms  avg tps: 53814.75

        kvbtree_u64_insert: run 10/10
        kvbtree_u64_insert:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0294ms  avg tps: 34035.82

        seeding btree: 10000/10000
        kvbtree_u64_insert_with_10k_existing_entries: run 10/10
        kvbtree_u64_insert_with_10k_existing_entries:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0379ms  avg tps: 26389.61

        kvbtree_u64_get_with_10k_existing_entries: run 10/10
        kvbtree_u64_get_with_10k_existing_entries:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0273ms  avg tps: 36693.72

        seeding btree: 100000/100000
        kvbtree_u64_insert_with_100k_existing_entries: run 10/10
        kvbtree_u64_insert_with_100k_existing_entries:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0504ms  avg tps: 19854.46

        kvbtree_u64_get_with_100k_existing_entries: run 10/10
        kvbtree_u64_get_with_100k_existing_entries:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0312ms  avg tps: 32024.93

        kvbtree_u64_remove_with_100k_existing_entries: run 10/10
        kvbtree_u64_remove_with_100k_existing_entries:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0524ms  avg tps: 19085.00

        seeding btree: 1000000/1000000
        kvbtree_u64_insert_with_1m_existing_entries: run 10/10
        kvbtree_u64_insert_with_1m_existing_entries:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0876ms  avg tps: 11419.71

        kvbtree_u64_get_with_1m_existing_entries: run 10/10
        kvbtree_u64_get_with_1m_existing_entries:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0387ms  avg tps: 25858.80

        kvmap_u64_set: run 10/10
        kvmap_u64_set:
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.0310ms  avg tps: 32235.40

        state_string_write(100kb): run 10/10
        state_string_write(100kb):
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.3717ms  avg tps: 2690.27

        kvvec_string_push(100kb): run 10/10
        kvvec_string_push(100kb):
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.6267ms  avg tps: 1595.69

        kvvecbtree_struct_insert(100kb): run 10/10
        kvvecbtree_struct_insert(100kb):
        10 runs, 1000 txs per run, total 10000
        avg per tx: 0.7200ms  avg tps: 1388.85
