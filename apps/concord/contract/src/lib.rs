const MAX_MESSAGE_LEN: usize = 4000;
const MAX_NAME_LEN: usize = 100;
const MAX_CHANNELS: usize = 50;

#[contract_state]
struct Contract {
    servers: KvVec<Server>,
    user_profiles: KvMap<Ed25519PublicKey, UserProfile>,
    dm_conversations: KvMap<DmKey, DmConversation>,
}

#[contract_methods]
impl Contract {
    #[authenticated]
    pub fn send_message(&mut self, server_id: u64, channel_id: u64, content: String) {
        let sender = message_sender();
        if content.len() > MAX_MESSAGE_LEN {
            return;
        }
        let server = self.servers.get(server_id).unwrap();

        let channel = server.get_channel(channel_id);

        let now = block_time();
        let msg = ChannelMessage {
            id: channel.messages.next_id(),
            content,
            author: sender,
            timestamp: now,
        };
        channel.messages.push(now, msg);
    }

    #[authenticated]
    pub fn delete_message(&mut self, server_id: u64, channel_id: u64, message_id: u64) {
        let sender = message_sender();
        let server = self.servers.get(server_id).unwrap();

        let is_server_owner = server.owner == sender;

        let channel = server.get_channel(channel_id);

        let msg = channel.messages.get(message_id).unwrap();

        let is_author = msg.author == sender;
        if !is_author && !is_server_owner {
            panic!("insufficient permissions");
        }

        channel.messages.remove(message_id);
    }

    #[authenticated]
    pub fn create_server(&mut self, name: String, encrypted_key: [u8; 64]) {
        let sender = message_sender();
        if name.len() > MAX_NAME_LEN {
            return;
        }
        let server_id = self.servers.length();

        let mut server = Server {
            id: server_id,
            name,
            owner: sender,
            members: KvBTree::default(),
            channels: vec![],
            next_channel_id: 0,
        };
        server.members.insert(sender, 0);
        server.add_channel("general".to_string());
        self.servers.push(server);

        let mut profile: UserProfile = self.user_profiles.get(&sender).unwrap_or_default();
        profile.encrypted_server_keys.push((server_id, encrypted_key));
        self.user_profiles.set(&sender, profile);
    }

    #[authenticated]
    pub fn join_server(&mut self, server_id: u64, encrypted_key: [u8; 64]) {
        let sender = message_sender();

        let server = self.servers.get(server_id).unwrap();

        if server.members.get(&sender).is_some() {
            return;
        }

        server.members.insert(sender, 0);

        let mut profile: UserProfile = self.user_profiles.get(&sender).unwrap_or_default();
        profile.encrypted_server_keys.push((server_id, encrypted_key));
        self.user_profiles.set(&sender, profile);
    }

    #[authenticated]
    pub fn leave_server(&mut self, server_id: u64) {
        let sender = message_sender();
        let server = self.servers.get(server_id).unwrap();

        if server.owner == sender {
            return;
        }

        server.members.remove(&sender);

        let mut profile: UserProfile = self.user_profiles.get(&sender).unwrap_or_default();
        profile.encrypted_server_keys.retain(|(id, _)| *id != server_id);
        self.user_profiles.set(&sender, profile);
    }

    #[authenticated]
    pub fn create_channel(&mut self, server_id: u64, name: String) {
        let sender = message_sender();
        if name.len() > MAX_NAME_LEN {
            return;
        }
        let mut server = self.servers.get(server_id).unwrap();

        if server.owner != sender {
            panic!("insufficient permissions");
        }

        server.add_channel(name);
        self.servers.set(server_id, server);
    }

    #[authenticated]
    pub fn delete_channel(&mut self, server_id: u64, channel_id: u64) {
        let sender = message_sender();
        let mut server = self.servers.get(server_id).unwrap();

        if server.owner != sender {
            panic!("insufficient permissions");
        }

        server.channels.retain(|c| c.id != channel_id);
        self.servers.set(server_id, server);
    }

    #[authenticated]
    pub fn set_display_name(&mut self, name: String) {
        let sender = message_sender();
        if name.len() > MAX_NAME_LEN {
            return;
        }
        let mut profile: UserProfile = self.user_profiles.get(&sender).unwrap_or_default();
        profile.display_name = Some(name);
        self.user_profiles.set(&sender, profile);
    }
    #[authenticated]
    pub fn kick_member(&mut self, server_id: u64, target: Ed25519PublicKey) {
        let sender = message_sender();

        let server = self.servers.get(server_id).unwrap();

        if server.owner != sender {
            panic!("insufficient permissions");
        }

        if target == server.owner {
            return;
        }

        server.members.remove(&target);
    }
    #[authenticated]
    pub fn send_dm(&mut self, recipient: Ed25519PublicKey, content: String) {
        let sender = message_sender();
        if content.len() > MAX_MESSAGE_LEN {
            return;
        }
        let key = dm_key(sender, recipient);
        let now = block_time();

        let mut convo = self.dm_conversations.get(&key).unwrap_or_default();

        if convo.messages.is_empty() {
            self.start_dm_conversation(&mut convo, &key, sender, recipient, now);
        } else {
            self.send_dm_in_active_conversation(&convo, &key, sender, recipient, now);
        }

        let msg =
            DmMessage { id: convo.messages.next_id(), content, author: sender, timestamp: now };
        convo.messages.push(now, msg);

        self.dm_conversations.set(&key, convo);
    }
    fn start_dm_conversation(
        &mut self,
        convo: &mut DmConversation,
        key: &DmKey,
        sender: Ed25519PublicKey,
        recipient: Ed25519PublicKey,
        now: u64,
    ) {
        let sender_profile: UserProfile = self.user_profiles.get(&sender).unwrap_or_default();
        let recipient_profile: UserProfile = self.user_profiles.get(&recipient).unwrap_or_default();

        let sender_aid = sender_profile.dm_activity.push(now, key.clone());
        let recipient_aid = recipient_profile.dm_activity.push(now, key.clone());

        self.user_profiles.set(&sender, sender_profile);
        self.user_profiles.set(&recipient, recipient_profile);

        if key.user_a == sender {
            convo.user_a_dm_activity_id = sender_aid;
            convo.user_b_dm_activity_id = recipient_aid;
        } else {
            convo.user_b_dm_activity_id = sender_aid;
            convo.user_a_dm_activity_id = recipient_aid;
        }
    }

    fn send_dm_in_active_conversation(
        &mut self,
        convo: &DmConversation,
        key: &DmKey,
        sender: Ed25519PublicKey,
        recipient: Ed25519PublicKey,
        now: u64,
    ) {
        let sender_is_a = key.user_a == sender;

        let sender_profile: UserProfile = self.user_profiles.get(&sender).unwrap_or_default();
        let sender_aid =
            if sender_is_a { convo.user_a_dm_activity_id } else { convo.user_b_dm_activity_id };
        sender_profile.dm_activity.update(sender_aid, now, key.clone());
        self.user_profiles.set(&sender, sender_profile);

        let recipient_profile: UserProfile = self.user_profiles.get(&recipient).unwrap_or_default();
        let recipient_aid =
            if sender_is_a { convo.user_b_dm_activity_id } else { convo.user_a_dm_activity_id };
        recipient_profile.dm_activity.update(recipient_aid, now, key.clone());
        self.user_profiles.set(&recipient, recipient_profile);
    }
    #[constructor]
    pub fn new(brotli_html_content: Vec<u8>) -> Self {
        runtime::register_static_route("", &brotli_html_content);
        return Self::default();
    }
}

#[contract_type]
struct ChannelMessage {
    id: u64,
    content: String,
    author: Ed25519PublicKey,
    timestamp: u64,
}

#[contract_type]
struct DmMessage {
    id: u64,
    content: String,
    author: Ed25519PublicKey,
    timestamp: u64,
}

#[contract_type]
struct Channel {
    id: u64,
    name: String,
    messages: KvVecBTree<u64, ChannelMessage>,
}

#[contract_type]
struct Server {
    id: u64,
    name: String,
    owner: Ed25519PublicKey,
    members: KvBTree<Ed25519PublicKey, u8>,
    channels: Vec<Channel>,
    next_channel_id: u64,
}

impl Server {
    fn add_channel(&mut self, name: String) {
        if self.channels.len() >= MAX_CHANNELS {
            return;
        }
        let id = self.next_channel_id;
        self.next_channel_id += 1;
        let channel = Channel { id, name, messages: KvVecBTree::default() };
        self.channels.push(channel);
    }

    fn get_channel(&self, id: u64) -> &Channel {
        let channel = self.channels.iter().find(|c| c.id == id).unwrap();
        return channel;
    }
}

#[contract_type]
struct DmConversation {
    messages: KvVecBTree<u64, DmMessage>,
    user_a_dm_activity_id: u64,
    user_b_dm_activity_id: u64,
}

#[contract_type]
struct DmKey {
    user_a: Ed25519PublicKey,
    user_b: Ed25519PublicKey,
}

#[contract_type]
struct UserProfile {
    dm_activity: KvVecBTree<u64, DmKey>,
    encrypted_server_keys: Vec<(u64, [u8; 64])>,
    display_name: Option<String>,
}

use vastrum_contract_macros::{
    authenticated, constructor, contract_methods, contract_state, contract_type,
};
use vastrum_runtime_lib::{
    Ed25519PublicKey, KvBTree, KvMap, KvVec, KvVecBTree,
    runtime::{block_time, message_sender},
};
mod utils;
use utils::*;
