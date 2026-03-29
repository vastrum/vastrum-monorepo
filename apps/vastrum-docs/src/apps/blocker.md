# Blocker

[Blocker](https://d66m4cniuqbgkeuetyvcbkfqfutt3qd3hdxdby2tlqifbt3otctq.vastrum.net)

Blocker is mostly just a fun prototype (Seemed cool to have a onchain chain explorer, a kind of ouroboros like challenge), final version of Vastrum will probably not support blocker.

Blocker is largely vibecoded.

Indexing currently decreases tps from 50k to roughly 10k for counter, this is partly because it is unoptimized but also for each transactions it adds 6 RocksDB writes + associated jmt state tree proofs updates.

The blockchain indexer on vastrum-node lives in [vastrum-node/src/block_indexer/indexer.rs (gitter preview)](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/vastrum-node/src/block_indexer/indexer.rs). Everytime a block is executed the indexer is called which writes metadata for every transaction executed in the block.

The frontend then reads that data from the keyvalue storage onchain to populate the pages such as account detail or transaction detail.

Data indexed
- Current block height
- Latest blocks
- Block details
- Latest transactions
- Sites deployed
- Sites details
- Account details
- Transaction detail




What that is interesting is that the data is written to a regular sitekv db, there is no special db for the indexed blockchain data. There is a constant site_id where all of the indexed blockchain writes are stored and then any application can query them.

```rust
pub fn indexed_blockchain_site_id() -> Sha256Digest {
    sha256_hash(b"indexed-blockchain-data-kv")
}
```

The data has the same state proofs as any other sitekv data.

[Blocker on Gitter](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/apps/blocker)