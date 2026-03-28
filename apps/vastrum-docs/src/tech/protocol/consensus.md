# Consensus


Vastrum uses a variant of the consensus protocol described [here](https://decentralizedthoughts.github.io/2025-06-18-simplex/)

Current testnet deployment is 8 validators (US,AUS,JAP,EU).

Initially the plan was to use minimmit however i did not like the large state space caused by pipelining. 

For example
-   1000s of blocks could be executed by one finalization if you have a long unfinalized chain of views
-   No guarantees about the state hash, how do you ensure blocks have execution state hash proofs?
-   Could speculatively execute unfinalized blocks and vote on state hash but then need to support execution reversion and complex multitimeline chain management


All of these problems can be solved, and doing pipelining is overall a better solution. However given the stage of the project i did not like the complexity.


Consensus is implemented in [vastrum-node/src/consensus/validator_state_machine.rs (gitter preview)](https://gitter.vastrum.net/repo/vastrum/tree/vastrum-node/src/consensus/validator_state_machine.rs).



## Madsim deterministic testing

I use madsim for deterministic testing of the consensus and P2P, madsim allows you to run "real" P2P test with real nodes + realistic lossy networking conditions with latency.

Test example

```rust
#[madsim::test]
async fn five_nodes_reach_consensus() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet::<5>();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    tokio::time::sleep(Duration::from_secs(360)).await;

    assert!(node_1.latest_finalized_height() >= 100, "network didn't reach consensus");

    assert_chain_consistency(&[node_1, node_2, node_3, node_4, node_5]);
}
```

You can also easily block network communication for nodes or kill nodes and later restart and test recovery of state from DB.

```rust
#[madsim::test]
async fn three_live_two_offline_network_does_not_reach_consensus() {
    let [node_1, node_2, _node_3, _node_4, node_5] = start_madsim_localnet();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    node_1.kill_node();
    node_2.kill_node();

    //remaining 3 should halt
    assert_node_is_halted(&node_5).await;
}
```

The greatest benefit is deterministic testing, if any Madsim test fails, you can rerun it with the same seed to exactly reproduce the error conditions. Otherwise it is very common to have flaky errors that are hard to debug "heisenbugs".


                                                            
    context: node=0 "madsim-main", task=4 (spawned at /home/./.cargo/registry/sr 
    c/index.crates.io-1949cf8c6b5b557f/madsim-0.2.34/src/sim/runtime/mod.rs:129:19) 
    note: run with `MADSIM_TEST_SEED=1773259308922493051` environment variable to   
    reproduce this error                                                            
    test minority_partition_halts_then_recovers ... FAILED      








### Links

This is an amazing talk about deterministic testing

[Testing a Single-Node, Single Threaded, Distributed System Written in 1985 By Will Wilson](https://www.youtube.com/watch?v=m3HwXlQPCEU)


The Commonware blog is also pretty good

[Commonware Runtime](https://commonware.xyz/blogs/commonware-runtime)



Link to madsim Github

[Madsim (Github)](https://github.com/madsim-rs/madsim)


Madsim testing code inside vastrum-node

[vastrum-node/tests/sim_consensus.rs](https://gitter.vastrum.net/repo/vastrum/tree/vastrum-node/tests/sim_consensus.rs)


---

*I based most of the sync implementation on Commonwares simplex implementation [commonware-consensus docs](https://docs.rs/commonware-consensus/latest/commonware_consensus/simplex/index.html)*