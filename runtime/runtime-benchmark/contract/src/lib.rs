use vastrum_contract_macros::{constructor, contract_methods, contract_state, contract_type};
use vastrum_runtime_lib::{KvBTree, KvMap, KvVec, KvVecBTree};

#[contract_type]
struct ForumPost {
    title: String,
    content: String,
    author: String,
    last_bump_time: u64,
    reply_count: u64,
}

#[contract_state]
struct Contract {
    state_u32: u32,
    state_string: String,
    kvvec_string: KvVec<String>,
    kvbtree_u64: KvBTree<u64, u64>,
    kvvecbtree_struct: KvVecBTree<u64, ForumPost>,
    kvmap_u64: KvMap<String, u64>,
}

#[contract_methods]
impl Contract {
    #[constructor]
    pub fn initialize() -> Self {
        Self::default()
    }

    pub fn add_to_counter(&mut self, amount: u32) {
        self.state_u32 += amount;
    }

    pub fn state_string_set(&mut self, value: String) {
        self.state_string = value;
    }

    pub fn kvvec_string_push(&mut self, value: String) {
        self.kvvec_string.push(value);
    }

    pub fn kvbtree_u64_insert(&mut self, key: u64, value: u64) {
        self.kvbtree_u64.insert(key, value);
    }

    pub fn kvbtree_u64_remove(&mut self, key: u64) {
        self.kvbtree_u64.remove(&key);
    }

    pub fn kvbtree_u64_get(&self, key: u64) -> Option<u64> {
        self.kvbtree_u64.get(&key)
    }

    pub fn kvvecbtree_struct_insert(&mut self, a: String, b: String, c: String) {
        let now = runtime::block_time();
        let entry =
            ForumPost { title: a, content: b, author: c, last_bump_time: now, reply_count: 0 };
        self.kvvecbtree_struct.push(now, entry);
    }

    pub fn kvmap_u64_set(&mut self, key: String, value: u64) {
        self.kvmap_u64.set(&key, value);
    }
}
