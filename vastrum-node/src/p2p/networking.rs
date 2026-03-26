pub struct Networking {
    peer_manager: Arc<PeerManager>,
    transaction_tx: UnboundedSender<Transaction>,
}

impl Networking {
    pub fn broadcast_proposal(&self, proposal: &Proposal) {
        self.peer_manager.broadcast_statement_to_all_peers(AppPayload::Proposal(proposal.clone()));
    }
    pub fn broadcast_vote(&self, vote: &ValidatorVote) {
        self.peer_manager.broadcast_statement_to_all_peers(AppPayload::Vote(vote.clone()));
    }
    pub fn broadcast_certificate(&self, cert: Certificate) {
        self.peer_manager.broadcast_statement_to_all_peers(AppPayload::Certificate(cert));
    }

    async fn handle_received_messages(
        mut message_rx: UnboundedReceiver<AppInboundMessage>,
        vote_tx: UnboundedSender<ValidatorVote>,
        proposal_tx: UnboundedSender<Proposal>,
        cert_tx: UnboundedSender<Certificate>,
        transaction_tx: UnboundedSender<Transaction>,
        current_round_for_sync: Arc<RwLock<RoundSyncStateExternal>>,
        db: Arc<Db>,
    ) {
        while let Some(msg) = message_rx.recv().await {
            match msg.payload {
                AppPayload::Vote(vote) => {
                    Networking::handle_vote(vote, &vote_tx);
                }
                AppPayload::Proposal(proposal) => {
                    Networking::handle_proposal(proposal, &proposal_tx);
                }
                AppPayload::GetSlotReq(request) => {
                    if let Some(respond) = msg.respond {
                        Networking::handle_get_slot_request(respond, request, db.clone());
                    }
                }
                AppPayload::GetRoundReq(request) => {
                    if let Some(respond) = msg.respond {
                        Networking::handle_get_round_request(
                            respond,
                            request,
                            current_round_for_sync.clone(),
                        );
                    }
                }
                AppPayload::TransactionGossip(transaction) => {
                    Self::ingest_transaction(transaction, &transaction_tx);
                }
                AppPayload::Certificate(cert) => {
                    let _ = cert_tx.send(cert);
                }
            }
        }
    }

    fn handle_vote(vote: ValidatorVote, vote_tx: &UnboundedSender<ValidatorVote>) {
        let _ = vote_tx.send(vote);
    }
    fn handle_proposal(proposal: Proposal, proposal_tx: &UnboundedSender<Proposal>) {
        let _ = proposal_tx.send(proposal);
    }
    fn ingest_transaction(
        transaction: Transaction,
        transaction_tx: &UnboundedSender<Transaction>,
    ) -> bool {
        let valid_signature = transaction.verify_signature();
        let valid_gas = transaction.verify_gas();
        if !valid_signature || !valid_gas {
            return false;
        }
        let _ = transaction_tx.send(transaction);
        return true;
    }

    pub async fn get_slot(&self, height: u64) -> Option<GetSlotReply> {
        let response = self
            .peer_manager
            .send_request_to_random_peer(AppPayload::GetSlotReq(GetSlotRequest { height }))
            .await?;
        GetSlotReply::decode(&response.payload).ok()
    }
    fn handle_get_slot_request(respond: ResponseHandle, request: GetSlotRequest, db: Arc<Db>) {
        tokio::spawn(async move {
            let slot = db.read_block(request.height);
            respond.respond(GetSlotReply { slot }.encode());
        });
    }

    pub async fn get_round(&self, height: u64, round: u64) -> Option<GetRoundReply> {
        let response = self
            .peer_manager
            .send_request_to_random_peer(AppPayload::GetRoundReq(GetRoundRequest { height, round }))
            .await?;
        GetRoundReply::decode(&response.payload).ok()
    }
    fn handle_get_round_request(
        respond: ResponseHandle,
        request: GetRoundRequest,
        current_round_for_sync: Arc<RwLock<RoundSyncStateExternal>>,
    ) {
        tokio::spawn(async move {
            let state = current_round_for_sync.read().await;

            let mut cert = None;
            if let Some(justify) = &state.latest_justify {
                let request_is_below_latest_justify = justify.round > request.round;
                if request_is_below_latest_justify {
                    cert = Some(Certificate::Justify(justify.clone()));
                }
            } else {
                let skip = state.skip_certs.get(&request.round);
                if let Some(skip) = skip {
                    cert = Some(Certificate::Skip(skip.clone()));
                }
            }

            respond.respond(GetRoundReply { cert }.encode());
        });
    }

    pub fn broadcast_transaction(&self, transaction: Transaction) {
        let was_valid_transaction =
            Self::ingest_transaction(transaction.clone(), &self.transaction_tx);
        if !was_valid_transaction {
            return;
        }
        let payload = AppPayload::TransactionGossip(transaction);
        self.peer_manager.broadcast_statement_to_all_peers(payload);
    }
    pub async fn start(
        vote_tx: UnboundedSender<ValidatorVote>,
        proposal_tx: UnboundedSender<Proposal>,
        cert_tx: UnboundedSender<Certificate>,
        transaction_tx: UnboundedSender<Transaction>,
        p2p_key: ed25519::PrivateKey,
        peers: Vec<KnownPeer>,
        current_round_for_sync: Arc<RwLock<RoundSyncStateExternal>>,
        db: Arc<Db>,
        epoch_state: &EpochState,
    ) -> Arc<Networking> {
        let (inbound_tx, inbound_rx) = mpsc::unbounded_channel::<AppInboundMessage>();

        let mut validator_p2p_keys = HashSet::new();
        for v in epoch_state.validator_data.values() {
            validator_p2p_keys.insert(v.p2p_key);
        }

        let peer_manager = PeerManager::start(inbound_tx, p2p_key, peers, validator_p2p_keys).await;

        let net = Arc::new(Networking {
            peer_manager: peer_manager.clone(),
            transaction_tx: transaction_tx.clone(),
        });

        tokio::spawn(async move {
            Networking::handle_received_messages(
                inbound_rx,
                vote_tx,
                proposal_tx,
                cert_tx,
                transaction_tx,
                current_round_for_sync,
                db,
            )
            .await;
        });

        net
    }
}

use crate::{
    consensus::types::{Certificate, Proposal, RoundSyncStateExternal, ValidatorVote},
    consensus::validator_state_machine::EpochState,
    db::Db,
    p2p::{
        connection::ResponseHandle,
        peer_manager::{KnownPeer, PeerManager},
        types::{
            app_types::{GetRoundReply, GetRoundRequest, GetSlotReply, GetSlotRequest},
            messages::AppInboundMessage,
            payload::AppPayload,
        },
    },
};
use vastrum_shared_types::{borsh::BorshExt, crypto::ed25519, types::execution::transaction::Transaction};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::{
    RwLock,
    mpsc::{self, UnboundedReceiver, UnboundedSender},
};
