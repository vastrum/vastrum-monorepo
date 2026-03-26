const INBOUND_CONVER: &str = "inbound_conversation";
const SENT_CONVERSATION: &str = "started_conversation";

pub struct ChatterState {
    //inbound conversation requets sends to this inbox
    inbound_conversation_requests_mailbox: ReadableAnonInbox<ChatRequest>,
    //used to keep track of sent conversation requests
    sent_conversation_requests_mailbox: ReadableAnonInbox<SentChatRequest>,

    //current username of user
    current_username: String,

    chat_mailboxes: HashMap<Sha256Digest, ReadableAnonInbox<ChatMessage>>,

    contract: ContractAbiClient,
}
impl ChatterState {
    pub async fn init() -> ChatterState {
        let inbound = ReadableAnonInbox::<ChatRequest>::create_mailbox(INBOUND_CONVER).await;
        let sent = ReadableAnonInbox::<SentChatRequest>::create_mailbox(SENT_CONVERSATION).await;
        let contract = ContractAbiClient::new(Sha256Digest::from_u64(0));

        return ChatterState {
            inbound_conversation_requests_mailbox: inbound,
            sent_conversation_requests_mailbox: sent,
            current_username: "".to_string(),
            chat_mailboxes: HashMap::new(),
            contract,
        };
    }
    //to start conversation
    //generate new shared private key for this conversation
    //generate two root_hashes used to generate new mailbox_id for each message
    //send conversation data to the mailbox of the user to start the conversation with
    //also store conversation data in local mailbox to be able to recover and sync
    //conversation state later (no local storage required)
    pub async fn start_conversation(&mut self, invitation_link: InvitationLink) {
        let contact_pub_key = invitation_link.pub_key;
        let contact_root = invitation_link.mailbox_root;

        //prevent duplicate conversations with same contact
        let sent =
            self.sent_conversation_requests_mailbox.get_messages_in_inbox(&self.contract).await;
        for request in &sent {
            let already_added_contact = request.contact_pub_key == contact_pub_key;
            if already_added_contact {
                return;
            }
        }

        let my_invite = get_invitation_link().await;
        let conversation_key = x25519::PrivateKey::from_rng();

        let initiator_root_hash = Sha256Digest::from_rng();
        let receiver_root_hash = Sha256Digest::from_rng();

        let from_username = self.current_username.clone();

        let request = ChatRequest {
            conversation_key: conversation_key.clone(),
            initiator_root_hash,
            receiver_root_hash,
            from_username,
            from_pub_key: my_invite.pub_key,
        };
        let mut receiver_inbox =
            AnonInbox::<ChatRequest>::import_inbox(contact_pub_key, contact_root);
        receiver_inbox.send_message_to_inbox(request.clone(), &self.contract).await;

        let sent_request = SentChatRequest {
            conversation_key,
            initiator_root_hash,
            receiver_root_hash,
            contact_mailbox_root: contact_root,
            contact_pub_key,
        };
        self.sent_conversation_requests_mailbox
            .send_message_to_inbox(sent_request, &self.contract)
            .await;
    }

    pub async fn send_message_in_conversation(
        &mut self,
        conversation: Conversation,
        content: String,
    ) {
        let contact_root = conversation.contact_root;

        let Some(conversation_inbox) = self.chat_mailboxes.get_mut(&contact_root) else {
            return;
        };

        let timestamp = Utc::now();
        let message = ChatMessage { content, timestamp };
        conversation_inbox.send_message_to_inbox(message, &self.contract).await;
    }

    //syncs all conversations
    //by checking inbound mailbox for all external conversation requests
    //and checking personal mailbox for all previously sent conversation requests by this identity
    pub async fn get_all_conversations(
        &mut self,
    ) -> HashMap<x25519::PublicKey, FrontendConversation> {
        let outbound =
            self.sent_conversation_requests_mailbox.get_messages_in_inbox(&self.contract).await;
        let inbound =
            self.inbound_conversation_requests_mailbox.get_messages_in_inbox(&self.contract).await;

        let mut conversations = vec![];

        for conversation in outbound {
            let my_root = conversation.initiator_root_hash;
            let contact_root = conversation.receiver_root_hash;
            let convo_key = conversation.conversation_key;
            let contact_mailbox_root = conversation.contact_mailbox_root;
            let contact_pub_key = conversation.contact_pub_key;
            let mut contact_name = self.get_name_from_mailbox_root(contact_mailbox_root).await;
            if contact_name.is_empty() {
                contact_name = contact_pub_key.to_string();
            }
            let conversation = Conversation {
                convo_key: convo_key.clone(),
                my_root,
                contact_root,
                contact_name,
                contact_pub_key,
            };
            conversations.push(conversation);

            //if chat mailboxes not yet stored in state, add them
            self.chat_mailboxes.entry(my_root).or_insert_with(|| {
                ReadableAnonInbox::<ChatMessage>::import_mailbox(convo_key.clone(), my_root)
            });
            self.chat_mailboxes.entry(contact_root).or_insert_with(|| {
                ReadableAnonInbox::<ChatMessage>::import_mailbox(convo_key, contact_root)
            });
        }

        for conversation in inbound {
            let my_root = conversation.receiver_root_hash;
            let contact_root = conversation.initiator_root_hash;
            let convo_key = conversation.conversation_key;
            let contact_pub_key = conversation.from_pub_key;
            let mut contact_name = conversation.from_username;
            if contact_name.is_empty() {
                contact_name = contact_pub_key.to_string();
            }
            let conversation = Conversation {
                convo_key: convo_key.clone(),
                my_root,
                contact_root,
                contact_name,
                contact_pub_key,
            };
            conversations.push(conversation);

            self.chat_mailboxes.entry(my_root).or_insert_with(|| {
                ReadableAnonInbox::<ChatMessage>::import_mailbox(convo_key.clone(), my_root)
            });

            self.chat_mailboxes.entry(contact_root).or_insert_with(|| {
                ReadableAnonInbox::<ChatMessage>::import_mailbox(convo_key, contact_root)
            });
        }

        let mut frontend_conversations = HashMap::new();
        for conversation in conversations {
            let messages = self.get_messages_for_conversation(&conversation).await;
            let contact_pub_key = conversation.contact_pub_key;
            let is_group = false;
            let frontend_conversation = FrontendConversation { conversation, messages, is_group };
            frontend_conversations.entry(contact_pub_key).or_insert(frontend_conversation);
        }

        return frontend_conversations;
    }

    async fn get_messages_for_conversation(
        &mut self,
        chat: &Conversation,
    ) -> Vec<ConversationMessage> {
        let sent = self.chat_mailboxes.get_mut(&chat.contact_root).unwrap();
        let sent_messages = sent.get_messages_in_inbox(&self.contract).await;
        let received = self.chat_mailboxes.get_mut(&chat.my_root).unwrap();
        let received_messages = received.get_messages_in_inbox(&self.contract).await;

        let mut conversation_messages = vec![];
        for message in sent_messages {
            let msg = ConversationMessage {
                from_me: true,
                content: message.content,
                timestamp: message.timestamp,
            };
            conversation_messages.push(msg);
        }

        for message in received_messages {
            let msg = ConversationMessage {
                from_me: false,
                content: message.content,
                timestamp: message.timestamp,
            };
            conversation_messages.push(msg);
        }

        return conversation_messages;
    }

    pub async fn get_name_from_mailbox_root(&self, mailbox_root: Sha256Digest) -> String {
        let hash_bytes = mailbox_root.encode();
        let salt_bytes = b"VASTRUM_NAME_SALT".to_vec();
        let bytes = [hash_bytes, salt_bytes].concat();
        let name_inbox_id = sha256_hash(&bytes);

        let key_value_content =
            self.contract.state().await.inbox.get(&name_inbox_id.to_string()).await;
        if key_value_content.is_none() {
            return "".to_string();
        }

        let private_key = x25519::PrivateKey::from_sha256_hash(mailbox_root);
        let cipher_text: CipherText = serde_json::from_str(&key_value_content.unwrap()).unwrap();
        let name = decrypt_string_x25519(cipher_text, &private_key);

        return name;
    }
    pub async fn set_name(&mut self, name: String) {
        let mailbox_root = get_mailbox_root().await;
        let hash_bytes = mailbox_root.encode();
        let salt_bytes = b"VASTRUM_NAME_SALT".to_vec();
        let bytes = [hash_bytes, salt_bytes].concat();
        let name_inbox_id = sha256_hash(&bytes);
        let private_key = x25519::PrivateKey::from_sha256_hash(mailbox_root);

        let cipher_text = encrypt_string_x25519(&name, &private_key, private_key.public_key());

        let content = serde_json::to_string(&cipher_text).unwrap();
        self.contract.write_to_inbox(name_inbox_id.to_string(), content).await;
        self.current_username = name;
    }
    pub async fn get_current_set_name(&mut self) -> String {
        let mailbox_root = get_mailbox_root().await;
        let name = self.get_name_from_mailbox_root(mailbox_root).await;
        self.current_username = name.clone();
        return name;
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub struct InvitationLink {
    pub pub_key: x25519::PublicKey,
    pub mailbox_root: Sha256Digest,
}

pub async fn get_mailbox_root() -> Sha256Digest {
    let mailbox_root = get_private_salt(INBOUND_CONVER.to_string()).await;
    return mailbox_root;
}
pub async fn get_invitation_link() -> InvitationLink {
    let private_key = get_private_salt(format!("private_key{INBOUND_CONVER}")).await;
    let private_key = x25519::PrivateKey::from_sha256_hash(private_key);
    let pub_key = private_key.public_key();
    let mailbox_root = get_mailbox_root().await;
    let link = InvitationLink { pub_key, mailbox_root };
    return link;
}
pub async fn get_conversation_invitation_link() -> String {
    let link = get_invitation_link().await;
    return encode_conversation_invitation_link(link.pub_key, link.mailbox_root);
}
pub fn encode_conversation_invitation_link(
    pub_key: x25519::PublicKey,
    mailbox_root: Sha256Digest,
) -> String {
    let link = InvitationLink { pub_key, mailbox_root };
    return bs58::encode(link.encode()).into_string();
}
pub fn parse_conversation_invitation_link(invitation_link: String) -> Option<InvitationLink> {
    let Ok(bytes) = bs58::decode(&invitation_link).into_vec() else { return None };
    let Ok(link) = InvitationLink::decode(&bytes) else { return None };
    return Some(link);
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatRequest {
    conversation_key: x25519::PrivateKey,
    initiator_root_hash: Sha256Digest,
    receiver_root_hash: Sha256Digest,
    from_username: String,
    from_pub_key: x25519::PublicKey,
}

#[derive(Serialize, Deserialize, Clone)]
struct SentChatRequest {
    conversation_key: x25519::PrivateKey,
    initiator_root_hash: Sha256Digest,
    receiver_root_hash: Sha256Digest,
    contact_mailbox_root: Sha256Digest,
    contact_pub_key: x25519::PublicKey,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    content: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conversation {
    pub convo_key: x25519::PrivateKey,
    my_root: Sha256Digest,
    contact_root: Sha256Digest,
    pub contact_name: String,
    pub contact_pub_key: x25519::PublicKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversationMessage {
    pub from_me: bool,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FrontendConversation {
    pub conversation: Conversation,
    pub messages: Vec<ConversationMessage>,
    pub is_group: bool,
}

use crate::chatter::anonymous_inbox::{AnonInbox, ReadableAnonInbox};
use borsh::{BorshDeserialize, BorshSerialize};
pub use chatter_abi::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vastrum_frontend_lib::get_private_salt;
use vastrum_shared_types::{
    borsh::BorshExt,
    crypto::{
        encryption::{CipherText, decrypt_string_x25519, encrypt_string_x25519},
        sha256::{Sha256Digest, sha256_hash},
        x25519,
    },
};

#[cfg(test)]
mod tests {

    use vastrum_shared_types::crypto::{sha256::Sha256Digest, x25519};

    use crate::chatter::chatter_state::{
        encode_conversation_invitation_link, parse_conversation_invitation_link,
    };

    #[test]
    fn test_encode_decode_invite_link() {
        let pub_key = x25519::PrivateKey::from_rng().public_key();
        let mailbox_root = Sha256Digest::from_rng();

        let link = encode_conversation_invitation_link(pub_key, mailbox_root);
        let parsed = parse_conversation_invitation_link(link).unwrap();
        assert_eq!(pub_key, parsed.pub_key);
        assert_eq!(mailbox_root, parsed.mailbox_root);
    }
}
