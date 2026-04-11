use std::collections::{BTreeMap, BTreeSet};
use vastrum_contract_macros::{
    authenticated, constructor, contract_methods, contract_state, contract_type,
};
use vastrum_runtime_lib::{Ed25519PublicKey, KvBTree, KvMap, KvVec, KvVecBTree};

#[contract_type]
struct UserData {
    name: String,
    visits: u64,
}

#[contract_type]
struct ForumPost {
    title: String,
    content: String,
    author: String,
    last_bump_time: u64,
    reply_count: u64,
}

#[contract_type]
struct NestedKvMapStruct {
    name: String,
    count: u64,
    inner: KvMap<String, u64>,
}

#[contract_type]
struct NestedKvVecStruct {
    name: String,
    inner: KvVec<String>,
}

#[contract_type]
struct NestedKvBTreeStruct {
    name: String,
    inner: KvBTree<u64, String>,
}

#[contract_type]
struct NestedKvVecBTreeStruct {
    name: String,
    inner: KvVecBTree<u64, ForumPost>,
}

#[contract_type]
enum TestEnum {
    AVariation,
    BVariation,
}

#[contract_type]
struct PrimitiveFields {
    flag_bool: bool,
    small_u: u8,
    mid_u: u16,
    big_u: u128,
    s8: i8,
    s16: i16,
    s32: i32,
    s64: i64,
    s128: i128,
    maybe_string: Option<String>,
    maybe_int: Option<u64>,
    numbers: Vec<u32>,
    fixed_bytes: [u8; 4],
    pair: (u64, String),
    btree: BTreeMap<String, u64>,
}

#[contract_state]
struct Contract {
    counter: u32,
    message: String,
    user_data: UserData,
    kvmap: KvMap<String, u64>,
    kvvec: KvVec<String>,
    kvvec_struct: KvVec<ForumPost>,
    kvbtree: KvBTree<u64, String>,
    kvvecbtree: KvVecBTree<u64, ForumPost>,
    nested_kvmap: KvMap<String, NestedKvMapStruct>,
    nested_kvvec: KvMap<String, NestedKvVecStruct>,
    nested_kvbtree: KvMap<String, NestedKvBTreeStruct>,
    nested_kvvecbtree: KvMap<String, NestedKvVecBTreeStruct>,
    last_authenticated_sender: Ed25519PublicKey,
    current_time: u64,
    enum_test: TestEnum,
    price_f32: f32,
    price_f64: f64,
    tag_set: BTreeSet<String>,
    int_set: BTreeSet<u64>,
    primitives: PrimitiveFields,
}

#[contract_methods]
impl Contract {
    #[constructor]
    pub fn initialize(initial_message: String) -> Self {
        Self { message: initial_message, ..Self::default() }
    }

    pub fn add_to_counter(&mut self, amount: u32) {
        self.counter += amount;
    }

    pub fn set_message(&mut self, msg: String) {
        self.message = msg;
    }

    pub fn set_user(&mut self, name: String, visits: u64) {
        self.user_data.name = name;
        self.user_data.visits = visits;
    }

    pub fn set_enum(&mut self, val: TestEnum) {
        self.enum_test = val;
    }

    pub fn set_timestamp(&mut self) {
        self.current_time = runtime::block_time();
    }

    pub fn get_sender(&self) {
        let sender = runtime::message_sender();
        runtime::log(&format!("Sender: {:02x?}", sender.bytes));
    }

    pub fn get_block_time(&self) {
        let time = runtime::block_time();
        runtime::log(&format!("Block time: {}", time));
    }

    pub fn add_page(&self, route: String, brotli_html_content: Vec<u8>) {
        runtime::register_static_route(&route, &brotli_html_content);
    }

    pub fn kvmap_set(&mut self, key: String, value: u64) {
        self.kvmap.set(&key, value);
    }

    pub fn kvmap_remove(&mut self, key: String) {
        self.kvmap.remove(&key);
    }

    pub fn kvmap_increment(&mut self, key: String) {
        let current = self.kvmap.get(&key).unwrap_or(0);
        self.kvmap.set(&key, current + 1);
    }

    pub fn kvvec_push(&mut self, value: String) {
        self.kvvec.push(value);
    }

    pub fn kvvec_set(&mut self, index: u64, value: String) {
        self.kvvec.set(index, value);
    }

    pub fn kvvec_struct_push(&mut self, title: String, content: String) {
        self.kvvec_struct.push(ForumPost {
            title,
            content,
            author: String::new(),
            last_bump_time: 0,
            reply_count: 0,
        });
    }

    pub fn kvbtree_insert(&mut self, key: u64, value: String) {
        self.kvbtree.insert(key, value);
    }

    pub fn kvbtree_remove(&mut self, key: u64) {
        self.kvbtree.remove(&key);
    }

    pub fn kvvecbtree_push(&mut self, title: String, content: String, author: String) {
        let now = runtime::block_time();
        let post = ForumPost { title, content, author, last_bump_time: now, reply_count: 0 };
        self.kvvecbtree.push(now, post);
    }

    pub fn kvvecbtree_update(&mut self, id: u64) {
        if let Some(mut post) = self.kvvecbtree.get(id) {
            let new_bump_time = runtime::block_time();
            post.last_bump_time = new_bump_time;
            post.reply_count += 1;
            self.kvvecbtree.update(id, new_bump_time, post);
        }
    }

    pub fn kvvecbtree_remove(&mut self, id: u64) {
        self.kvvecbtree.remove(id);
    }

    pub fn nested_kvvec_create(&mut self, key: String, name: String) {
        self.nested_kvvec.set(&key, NestedKvVecStruct { name, inner: KvVec::new() });
    }

    pub fn nested_kvvec_push(&mut self, outer_key: String, value: String) {
        if let Some(s) = self.nested_kvvec.get(&outer_key) {
            s.inner.push(value);
        }
    }

    pub fn nested_kvbtree_create(&mut self, key: String, name: String) {
        self.nested_kvbtree.set(&key, NestedKvBTreeStruct { name, inner: KvBTree::new() });
    }

    pub fn nested_kvbtree_insert(&mut self, outer_key: String, key: u64, value: String) {
        if let Some(s) = self.nested_kvbtree.get(&outer_key) {
            s.inner.insert(key, value);
        }
    }

    pub fn nested_kvbtree_remove(&mut self, outer_key: String, key: u64) {
        if let Some(s) = self.nested_kvbtree.get(&outer_key) {
            s.inner.remove(&key);
        }
    }

    pub fn nested_kvvecbtree_create(&mut self, key: String, name: String) {
        self.nested_kvvecbtree.set(&key, NestedKvVecBTreeStruct { name, inner: KvVecBTree::new() });
    }

    pub fn nested_kvvecbtree_push(&mut self, outer_key: String, title: String, content: String) {
        if let Some(s) = self.nested_kvvecbtree.get(&outer_key) {
            let now = runtime::block_time();
            let post = ForumPost {
                title,
                content,
                author: String::new(),
                last_bump_time: now,
                reply_count: 0,
            };
            s.inner.push(now, post);
        }
    }

    pub fn nested_kvvecbtree_remove(&mut self, outer_key: String, id: u64) {
        if let Some(s) = self.nested_kvvecbtree.get(&outer_key) {
            s.inner.remove(id);
        }
    }

    pub fn nested_kvmap_create(&mut self, key: String, name: String) {
        self.nested_kvmap.set(&key, NestedKvMapStruct { name, count: 0, inner: KvMap::new() });
    }

    pub fn nested_kvmap_set(&mut self, outer_key: String, inner_key: String, value: u64) {
        if let Some(mut s) = self.nested_kvmap.get(&outer_key) {
            s.inner.set(&inner_key, value);
            s.count += 1;
            self.nested_kvmap.set(&outer_key, s);
        }
    }

    pub fn nested_kvmap_remove(&mut self, outer_key: String, inner_key: String) {
        if let Some(mut s) = self.nested_kvmap.get(&outer_key) {
            s.inner.remove(&inner_key);
            if s.count > 0 {
                s.count -= 1;
            }
            self.nested_kvmap.set(&outer_key, s);
        }
    }

    pub fn kv_insert_raw(&mut self, key: String, value: Vec<u8>) {
        runtime::kv_insert(&format!("n.raw.{}", key), &value);
    }

    pub fn kv_delete_raw(&mut self, key: String) {
        runtime::kv_delete(&format!("n.raw.{}", key));
    }

    pub fn kv_check_raw_exists(&mut self, key: String) {
        let val = runtime::kv_get(&format!("n.raw.{}", key));
        self.message = if val.is_empty() { String::new() } else { "exists".to_string() };
    }

    pub fn write_then_panic(&mut self, key: String, value: u64) {
        self.kvmap.set(&key, value);
        self.counter += 1;
        self.kvvec.push(format!("should not persist: {key}"));
        panic!("intentional panic after writes");
    }

    #[authenticated]
    pub fn auth_record_sender(&mut self) {
        self.last_authenticated_sender = runtime::message_sender();
    }

    pub fn set_price_f32(&mut self, v: f32) {
        self.price_f32 = v;
    }

    pub fn set_price_f64(&mut self, v: f64) {
        self.price_f64 = v;
    }

    pub fn insert_tag(&mut self, tag: String) {
        self.tag_set.insert(tag);
    }

    pub fn remove_tag(&mut self, tag: String) {
        self.tag_set.remove(&tag);
    }

    pub fn insert_int(&mut self, v: u64) {
        self.int_set.insert(v);
    }

    pub fn remove_int(&mut self, v: u64) {
        self.int_set.remove(&v);
    }

    pub fn set_bool(&mut self, v: bool) {
        self.primitives.flag_bool = v;
    }

    pub fn set_u8(&mut self, v: u8) {
        self.primitives.small_u = v;
    }

    pub fn set_u16(&mut self, v: u16) {
        self.primitives.mid_u = v;
    }

    pub fn set_u128(&mut self, v: u128) {
        self.primitives.big_u = v;
    }

    pub fn set_i8(&mut self, v: i8) {
        self.primitives.s8 = v;
    }

    pub fn set_i16(&mut self, v: i16) {
        self.primitives.s16 = v;
    }

    pub fn set_i32(&mut self, v: i32) {
        self.primitives.s32 = v;
    }

    pub fn set_i64(&mut self, v: i64) {
        self.primitives.s64 = v;
    }

    pub fn set_i128(&mut self, v: i128) {
        self.primitives.s128 = v;
    }

    pub fn set_maybe_string(&mut self, v: Option<String>) {
        self.primitives.maybe_string = v;
    }

    pub fn set_maybe_int(&mut self, v: Option<u64>) {
        self.primitives.maybe_int = v;
    }

    pub fn push_number(&mut self, v: u32) {
        self.primitives.numbers.push(v);
    }

    pub fn clear_numbers(&mut self) {
        self.primitives.numbers.clear();
    }

    pub fn set_fixed_bytes(&mut self, v: [u8; 4]) {
        self.primitives.fixed_bytes = v;
    }

    pub fn set_pair(&mut self, a: u64, b: String) {
        self.primitives.pair = (a, b);
    }

    pub fn btree_insert(&mut self, k: String, v: u64) {
        self.primitives.btree.insert(k, v);
    }

    pub fn btree_remove(&mut self, k: String) {
        self.primitives.btree.remove(&k);
    }

}
