const MAX_NAME_LEN: usize = 100;
const MAX_TITLE_LEN: usize = 200;
const MAX_DESCRIPTION_LEN: usize = 512;
const MAX_CONTENT_LEN: usize = 128000;

#[contract_state]
struct Contract {
    categories: KvMap<String, Category>,
    category_list: Vec<String>,
    admins: Vec<Ed25519PublicKey>,
}

#[contract_methods]
impl Contract {
    #[authenticated]
    pub fn create_post(&mut self, category_name: String, title: String, content: String) {
        if title.len() > MAX_TITLE_LEN || content.len() > MAX_CONTENT_LEN {
            return;
        }
        let now = runtime::block_time();
        let from = runtime::message_sender();

        let category = self.categories.get(&category_name).unwrap();
        let id = category.posts.next_id();
        let post = Post {
            id,
            title,
            content,
            timestamp: now,
            last_bump_time: now,
            replies: KvVecBTree::default(),
            from,
        };
        category.posts.push(now, post);
    }

    #[authenticated]
    pub fn reply_to_post(&mut self, category_name: String, post_id: u64, content: String) {
        if content.len() > MAX_CONTENT_LEN {
            return;
        }
        let category = self.categories.get(&category_name).unwrap();
        let mut post = category.posts.get(post_id).unwrap();

        let now = runtime::block_time();
        let from = runtime::message_sender();
        let reply_id = post.replies.next_id();
        let reply = PostReply { id: reply_id, content, timestamp: now, from };

        post.last_bump_time = now;

        post.replies.push(now, reply);

        category.posts.update(post_id, now, post);
    }

    #[authenticated]
    pub fn create_category(&mut self, name: String, description: String) {
        self.require_admin();
        if name.len() > MAX_NAME_LEN || description.len() > MAX_DESCRIPTION_LEN {
            return;
        }
        if self.categories.contains(&name) {
            return;
        }
        let category = Category { name: name.clone(), description, posts: KvVecBTree::default() };
        self.categories.set(&name, category);
        self.category_list.push(name);
    }

    #[authenticated]
    pub fn delete_post(&mut self, category_name: String, post_id: u64) {
        self.require_admin();
        let category = self.categories.get(&category_name).unwrap();
        category.posts.remove(post_id);
    }

    #[authenticated]
    pub fn delete_reply(&mut self, category_name: String, post_id: u64, reply_id: u64) {
        self.require_admin();
        let category = self.categories.get(&category_name).unwrap();
        let post = category.posts.get(post_id).unwrap();

        post.replies.remove(reply_id);

        category.posts.update(post_id, runtime::block_time(), post);
    }
    fn require_admin(&self) {
        let sender = runtime::message_sender();
        if !self.admins.contains(&sender) {
            panic!("not a admin");
        }
    }

    #[constructor]
    pub fn new(brotli_html_content: Vec<u8>, initial_admin: Ed25519PublicKey) -> Self {
        runtime::register_static_route("", &brotli_html_content);
        Self { admins: vec![initial_admin], ..Self::default() }
    }
}

#[contract_type]
struct Post {
    id: u64,
    title: String,
    content: String,
    timestamp: u64,
    last_bump_time: u64,
    replies: KvVecBTree<u64, PostReply>,
    from: Ed25519PublicKey,
}
#[contract_type]
struct PostReply {
    id: u64,
    content: String,
    timestamp: u64,
    from: Ed25519PublicKey,
}

#[contract_type]
struct Category {
    name: String,
    description: String,
    posts: KvVecBTree<u64, Post>,
}

use vastrum_contract_macros::{authenticated, constructor, contract_methods, contract_state, contract_type};
use vastrum_runtime_lib::{Ed25519PublicKey, KvMap, KvVecBTree};
