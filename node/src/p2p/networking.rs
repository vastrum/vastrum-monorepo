impl Networking {
    pub async fn run(
        &self,
        addr: SocketAddr,
        peers: Vec<NodeRecord>,
        database: Arc<BlockchainDatabase>,
    ) {
        let p2p_domon = self.p2p_daemon.clone();

        tokio::spawn(async move {
            p2p_domon.run(addr, peers).await;
        });

        let message_request_rx = self.message_request_rx.clone();
        let block_vote_tx = self.block_vote_tx.clone();
        let null_vote_tx = self.null_vote_tx.clone();
        let proposedblock_tx = self.proposedblock_tx.clone();
        let transaction_tx = self.transaction_tx.clone();
        let p2p_daemon = self.p2p_daemon.clone();
        tokio::spawn(async move {
            Networking::handle_received_messages(
                message_request_rx,
                block_vote_tx,
                null_vote_tx,
                proposedblock_tx,
                transaction_tx,
                p2p_daemon,
                database,
            )
            .await;
        });
    }
    pub fn broadcast_block_proposal(&self, block: &Block) {
        let content = block.encode();
        let payload = Payload { payload_type: PayloadType::BlockProposal, content: content };
        let bytes = payload.encode();
        self.p2p_daemon.broadcast_statement_to_all_peers(bytes.to_vec());
    }
    pub fn broadcast_null_vote(&self, null_vote: &NullVote) {
        let content = null_vote.encode();
        let payload = Payload { payload_type: PayloadType::VoteNull, content: content };
        let bytes = payload.encode();
        self.p2p_daemon.broadcast_statement_to_all_peers(bytes.to_vec());
    }
    pub fn broadcast_block_vote(&self, block_vote: &BlockVote) {
        let content = block_vote.encode();
        let payload = Payload { payload_type: PayloadType::VoteBlock, content: content };
        let bytes = payload.encode();
        self.p2p_daemon.broadcast_statement_to_all_peers(bytes.to_vec());
    }
    pub async fn handle_received_messages(
        message_rx: Arc<Mutex<UnboundedReceiver<ReceivedMessage>>>,
        block_vote_tx: UnboundedSender<BlockVote>,
        null_vote_tx: UnboundedSender<NullVote>,
        proposedblock_tx: UnboundedSender<Block>,
        transaction_tx: UnboundedSender<Transaction>,
        p2p_daemon: Arc<P2PDaemon>,
        database: Arc<BlockchainDatabase>,
    ) {
        let mut message_rx = message_rx.lock().await;

        loop {
            let database = database.clone();
            let Some(received_msg) = message_rx.recv().await else {
                break;
            };
            let message = received_msg.message.clone();
            let payload = Payload::decode(&message.payload).unwrap();

            match payload.payload_type {
                PayloadType::VoteBlock => {
                    Networking::handle_vote_block_message(payload.content, block_vote_tx.clone());
                }
                PayloadType::VoteNull => {
                    Networking::handle_vote_null_message(payload.content, null_vote_tx.clone());
                }
                PayloadType::BlockProposal => {
                    Networking::handle_proposed_block_message(
                        payload.content,
                        proposedblock_tx.clone(),
                    );
                }
                PayloadType::GetNotarizationRequest => {
                    Networking::handle_get_notarization_request(
                        received_msg,
                        payload.content,
                        database,
                    )
                    .await
                }
                PayloadType::GetBlockRequest => {
                    Networking::handle_get_100_block_request(
                        received_msg,
                        payload.content,
                        database,
                    )
                    .await
                }
                PayloadType::GetPeersRequest => {
                    Networking::handle_get_peers_request(received_msg, p2p_daemon.clone()).await
                }
                PayloadType::TransactionGossip => {
                    Networking::handle_new_transaction_gossip(
                        received_msg,
                        payload.content,
                        transaction_tx.clone(),
                    );
                }
                PayloadType::TransactionSubmitBroadcast => {
                    Networking::handle_transaction_rebroadcast(
                        received_msg,
                        payload.content,
                        transaction_tx.clone(),
                        p2p_daemon.clone(),
                    )
                }
                _ => (),
            }
        }
    }
    pub fn handle_vote_block_message(content: Vec<u8>, block_vote_tx: UnboundedSender<BlockVote>) {
        let block_vote = BlockVote::decode(&content).unwrap();
        block_vote_tx.send(block_vote).unwrap();
    }
    pub fn handle_vote_null_message(content: Vec<u8>, null_vote_tx: UnboundedSender<NullVote>) {
        let null_vote = NullVote::decode(&content).unwrap();
        null_vote_tx.send(null_vote).unwrap();
    }
    pub fn handle_proposed_block_message(
        content: Vec<u8>,
        proposedblock_tx: UnboundedSender<Block>,
    ) {
        let proposed_block = Block::decode(&content).unwrap();
        proposedblock_tx.send(proposed_block).unwrap();
    }
    pub async fn get_100_block_from_height(&self, height: u64) -> Vec<Block> {
        let content = GetBlockRequest { slot_height: height };

        let request =
            Payload { payload_type: PayloadType::GetBlockRequest, content: content.encode() };

        let payload = request.encode();
        let response = self.p2p_daemon.send_request_to_random_peer(payload).await;

        if let Ok(message) = response {
            let content = message.payload;

            let payload = Payload::decode(&content);
            if let Ok(payload) = payload {
                let reply = GetBlockReply::decode(&payload.content);
                if let Ok(reply) = reply {
                    return reply.blocks;
                }
            }
        }
        return vec![];
    }
    pub async fn handle_get_100_block_request(
        received_message: ReceivedMessage,
        content: Vec<u8>,
        database: Arc<BlockchainDatabase>,
    ) {
        let request = GetBlockRequest::decode(&content).unwrap();

        let mut slots = vec![];
        for i in request.slot_height..=(request.slot_height + 100) {
            let slot_data = database.read_slot(i);
            if let Some(slot_data) = slot_data {
                slots.push(slot_data);
            }
        }

        let mut blocks = vec![];
        for slot in slots {
            if let SlotState::Block(notarized_block) = slot {
                let block = Block {
                    height: notarized_block.height,
                    transactions: notarized_block.transactions,
                    previous_block_hash: notarized_block.previous_block_hash,
                    slot_leader_signature: notarized_block.slot_leader_signature,
                };
                blocks.push(block);
            }
        }
        let reply = GetBlockReply { blocks: blocks };
        let response =
            Payload { payload_type: PayloadType::GetBlockReply, content: reply.encode() };
        let payload = response.encode();

        let _ = received_message
            .from_peer_connection
            .send_response(payload.to_vec(), received_message.message.id)
            .await;
    }
    pub async fn get_100_notarization(&self, height: u64) -> Option<GetNotarizationReply> {
        let content = GetNotarizationRequest { slot_height: height };

        let request = Payload {
            payload_type: PayloadType::GetNotarizationRequest,
            content: content.encode(),
        };
        let payload = request.encode();
        let response = self.p2p_daemon.send_request_to_random_peer(payload).await;

        if let Ok(message) = response {
            let content = message.payload;
            let payload = Payload::decode(&content);
            if let Ok(payload) = payload {
                let reply = GetNotarizationReply::decode(&payload.content);
                if let Ok(reply) = reply {
                    return Some(reply);
                }
            }
        }
        return None;
    }
    pub async fn handle_get_notarization_request(
        received_message: ReceivedMessage,
        content: Vec<u8>,
        database: Arc<BlockchainDatabase>,
    ) {
        let request = GetNotarizationRequest::decode(&content).unwrap();

        let mut slots = vec![];
        for i in request.slot_height..=(request.slot_height + 100) {
            let slot_data = database.read_slot(i);
            if let Some(slot_data) = slot_data {
                slots.push(slot_data);
            }
        }

        let mut notarizations = vec![];
        for slot in slots {
            if let SlotState::Block(notarized_block) = slot {
                let hash = notarized_block.calculate_block_hash();
                let notarization = GetNotarizationState {
                    height: notarized_block.height,
                    votes: notarized_block.votes,
                    hash: hash,
                    notarization_type: GetNotarizationNotarizationType::Block,
                };
                notarizations.push(notarization);
            } else if let SlotState::Nullification(nullification) = slot {
                let notarization = GetNotarizationState {
                    height: nullification.height,
                    votes: nullification.votes,
                    hash: ValidatorStateMachine::calculate_null_hash(nullification.height),
                    notarization_type: GetNotarizationNotarizationType::Nullification,
                };
                notarizations.push(notarization);
            }
        }

        let reply = GetNotarizationReply { notarizations: notarizations };
        let response =
            Payload { payload_type: PayloadType::GetNotarizationReply, content: reply.encode() };
        let payload = response.encode();

        let _ = received_message
            .from_peer_connection
            .send_response(payload, received_message.message.id)
            .await;
    }
    pub async fn handle_get_peers_request(
        received_message: ReceivedMessage,
        p2p_daemon: Arc<P2PDaemon>,
    ) {
        p2p_daemon
            .handle_peer_discovery_request(
                received_message.from_peer_connection,
                received_message.message.id,
            )
            .await;
    }
    pub fn handle_new_transaction_gossip(
        _received_message: ReceivedMessage,
        content: Vec<u8>,
        transaction_tx: UnboundedSender<Transaction>,
    ) {
        let transaction_gossip = TransactionGossip::decode(&content).unwrap();
        let transaction = transaction_gossip.transaction;
        let _ = transaction_tx.send(transaction);
    }
    pub fn handle_transaction_rebroadcast(
        _received_message: ReceivedMessage,
        content: Vec<u8>,
        transaction_tx: UnboundedSender<Transaction>,
        p2p_daemon: Arc<P2PDaemon>,
    ) {
        //todo add dos protection here
        //check if valid transaction and has not been included in current view
        //also does not have this tx in mempool currently (first time receiving this tx)

        let transaction_gossip = TransactionGossip::decode(&content).unwrap();
        let _ = transaction_tx.send(transaction_gossip.transaction.clone());

        //now rebroadcast transaction to all connected nodes
        let payload = Payload {
            payload_type: PayloadType::TransactionGossip,
            content: transaction_gossip.encode(),
        };
        let bytes = payload.encode();
        p2p_daemon.broadcast_statement_to_all_peers(bytes.to_vec());
    }
    pub fn broadcast_transaction(&self, transaction: Transaction) {
        let transaction_gossip = TransactionGossip { transaction: transaction };
        //todo add dos protection here
        let payload = Payload {
            payload_type: PayloadType::TransactionGossip,
            content: transaction_gossip.encode(),
        };
        let bytes = payload.encode();
        let _ = self.transaction_tx.send(transaction_gossip.transaction.clone());
        self.p2p_daemon.broadcast_statement_to_all_peers(bytes.to_vec());
    }
    pub fn new(
        block_vote_tx: UnboundedSender<BlockVote>,
        null_vote_tx: UnboundedSender<NullVote>,
        proposedblock_tx: UnboundedSender<Block>,
        transaction_tx: UnboundedSender<Transaction>,
        p2p_key: ed25519::PrivateKey,
        port: u16,
    ) -> Networking {
        let (message_request_tx, message_request_rx) = mpsc::unbounded_channel::<ReceivedMessage>();

        return Networking {
            p2p_daemon: Arc::new(P2PDaemon::new(message_request_tx, p2p_key, port)),
            block_vote_tx: block_vote_tx,
            null_vote_tx,
            proposedblock_tx: proposedblock_tx,
            transaction_tx: transaction_tx,
            message_request_rx: Arc::new(Mutex::new(message_request_rx)),
        };
    }
}

#[derive(Debug)]
pub struct Networking {
    p2p_daemon: Arc<P2PDaemon>,
    block_vote_tx: UnboundedSender<BlockVote>,
    null_vote_tx: UnboundedSender<NullVote>,
    proposedblock_tx: UnboundedSender<Block>,
    transaction_tx: UnboundedSender<Transaction>,
    message_request_rx: Arc<Mutex<UnboundedReceiver<ReceivedMessage>>>,
}

use crate::{
    consensus::{
        comms::{BlockVote, NullVote},
        minimmit::ValidatorStateMachine,
        types::{Block, SlotState},
    },
    db::blockchain::BlockchainDatabase,
    p2p::{
        domon::{NodeRecord, P2PDaemon},
        types::{
            getblock::{GetBlockReply, GetBlockRequest},
            getnotarization::{
                GetNotarizationNotarizationType, GetNotarizationReply, GetNotarizationRequest,
                GetNotarizationState,
            },
            messages::ReceivedMessage,
            payload::{Payload, PayloadType},
            transactiongossip::TransactionGossip,
        },
    },
};
use shared_types::{borsh::BorshExt, crypto::ed25519, types::execution::transaction::Transaction};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{
    Mutex,
    mpsc::{self, UnboundedReceiver, UnboundedSender},
};
