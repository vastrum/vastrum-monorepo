mod benchmarks;
mod helpers;

pub const TXS_PER_BATCH: usize = 1000;
pub const RUNS: usize = 10;

fn main() {
    benchmarks::counter::run();
    benchmarks::kvbtree::run();
    benchmarks::kvmap::run();
    benchmarks::state_writes::run();
}
