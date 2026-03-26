/*
 ,_     _
 |\\_,-~/
 / _  _ |    ,--.
(  @  @ )   / ,-'
 \  _T_/-._( (
 /         `. \
|         _  \ |
 \ \ ,  /      |
  || |-_\__   /
 ((_/`(____,-'
*/
//consensus based on this https://decentralizedthoughts.github.io/2025-06-18-simplex/
impl ValidatorStateMachine {
    pub async fn start_node(db: Arc<Db>, config: NodeConfig) {
        let mut validator = ValidatorStateMachine::initialize(db, config).await;
        validator.run().await;
    }
    async fn run(&mut self) {
        loop {
            self.process_consensus_messages();

            self.validator_actor();

            self.progress_consensus_round();
            self.progress_blockchain();

            self.process_mempool();

            self.recover_sync();
            sleep(Duration::from_millis(1)).await;
        }
    }
    fn validator_actor(&mut self) {
        let is_leader = self.is_leader(self.pub_key, self.current_height, self.current_round);
        let is_validator = self.has_stake_at_height(self.pub_key, self.current_height);

        if is_leader {
            self.execute_leader_role();
        }
        if is_validator {
            self.execute_validator_role();
        }
    }
    fn execute_leader_role(&mut self) {
        let state = self.local_validator_state(self.current_height, self.current_round);
        let has_not_proposed_block = state.leader_state == LeaderState::HasNotProposedBlock;

        if has_not_proposed_block {
            let Some(block_to_propose) = self.get_block_to_propose() else {
                return;
            };
            let state = self.local_validator_state(self.current_height, self.current_round);
            state.leader_state = LeaderState::HasProposedBlock;
            self.networking.broadcast_proposal(&block_to_propose);
            self.handle_proposal_received(block_to_propose);
        }
    }
    fn execute_validator_role(&mut self) {
        let height = self.current_height;
        let round = self.current_round;

        let validator_index = self.validator_index(self.pub_key, height).unwrap();
        let state = self.local_validator_state(height, round);

        let has_not_justify_voted = state.justify_state == JustifyState::NoVoteYet;
        if has_not_justify_voted {
            let proposal = self.get_valid_proposal(height, round);
            let has_proposal = proposal.is_some();
            if has_proposal {
                let proposal = proposal.unwrap();
                let block_hash = proposal.block.calculate_hash();

                let vote = ValidatorVote::create_signed(
                    VoteType::Justify(block_hash),
                    height,
                    round,
                    validator_index,
                    &self.private_key,
                );

                let state = self.local_validator_state(height, round);
                state.justify_state = JustifyState::VotedFor(block_hash);

                self.db.write_ahead_log_justify_vote(height, round, proposal.block);
                self.networking.broadcast_vote(&vote);
                self.handle_vote(vote);
            }
        }

        let state = self.local_validator_state(height, round);
        let has_not_commit_voted = state.commit_state == CommitState::NoVoteYet;

        let deadline_expired;

        //if high round, assume timeout is too low, decrease it.
        let faulty_slot = round > 5;
        if faulty_slot {
            deadline_expired = self.entered_round_at + LONG_ROUND_TIMEOUT <= Instant::now()
        } else {
            deadline_expired = self.entered_round_at + ROUND_TIMEOUT <= Instant::now()
        }
        if has_not_commit_voted {
            let round_state = self.round_state(height, round);
            let exists_justified_block = round_state.justify_cert.is_some();

            if exists_justified_block {
                //safe to vote on justified block even if do not have backing block locally
                //as know at least 33% honest nodes have the full block locally
                //because honest nodes will only justify block if they have it locally

                let justify_cert = round_state.justify_cert.as_ref().unwrap();
                let block_hash = justify_cert.block_hash;

                let vote = ValidatorVote::create_signed(
                    VoteType::Finalize(block_hash),
                    height,
                    round,
                    validator_index,
                    &self.private_key,
                );
                let state = self.local_validator_state(height, round);
                state.commit_state = CommitState::Finalize(block_hash);

                self.db.write_ahead_log_finalize_vote(height, round, block_hash);
                self.networking.broadcast_vote(&vote);
                self.handle_vote(vote);
            } else if deadline_expired {
                let vote = ValidatorVote::create_signed(
                    VoteType::Skip,
                    height,
                    round,
                    validator_index,
                    &self.private_key,
                );
                let state = self.local_validator_state(height, round);
                state.commit_state = CommitState::Skip;

                self.db.write_ahead_log_skip_vote(height, round);
                self.networking.broadcast_vote(&vote);
                self.handle_vote(vote);
            }
        }
    }

    fn process_consensus_messages(&mut self) {
        while let Ok(vote) = self.vote_rx.try_recv() {
            self.handle_vote(vote);
        }
        while let Ok(block_proposal) = self.proposal_rx.try_recv() {
            self.handle_proposal_received(block_proposal);
        }
        while let Ok(block) = self.block_sync_rx.try_recv() {
            self.handle_sync_finalized_block_received(block);
        }
        while let Ok(cert) = self.cert_rx.try_recv() {
            match cert {
                Certificate::Justify(c) => self.handle_justify_cert_received(c),
                Certificate::Skip(c) => self.handle_skip_cert_received(c),
                Certificate::Finalization(c) => self.handle_finalization_cert_received(c),
            }
        }
    }
    fn handle_vote(&mut self, vote: ValidatorVote) -> HandleVoteResult {
        let height = vote.height;
        let round = vote.round;
        let signature = vote.signature;
        let validator_index = vote.validator_index;

        if vote.height >= self.current_height + MAX_SLOT_LOOKAHEAD {
            return HandleVoteResult::ErrorTooFarIntoFuture;
        }
        if vote.round > self.current_round + MAX_ROUND_LOOKAHEAD {
            return HandleVoteResult::ErrorTooFarIntoFuture;
        }

        let Some(epoch_state) = self.epoch_state(height) else {
            return HandleVoteResult::ErrorEpochStateDoesNotYetExistForSlot;
        };

        let Some(validator) = epoch_state.validator_data(validator_index).copied() else {
            return HandleVoteResult::ErrorCouldNotFindValidatorIndex;
        };
        let is_valid_signature = validator.pub_key.verify_sig(vote.hash(), signature);

        if !is_valid_signature {
            return HandleVoteResult::ErrorInvalidSignature;
        }
        let validator_state = self.validator_state(height, round, validator_index);

        match vote.vote {
            VoteType::Justify(block_hash) => {
                let already_voted = validator_state.justify_state != JustifyState::NoVoteYet;
                if already_voted {
                    return HandleVoteResult::ErrorValidatorHasAlreadyVotedInThisSlotRound;
                }

                validator_state.justify_state = JustifyState::VotedFor(block_hash);

                let block = self.get_block_candidate(height, round, &block_hash);
                block.justify_votes.insert(validator_index, signature);
                block.justify_stake += validator.stake;

                self.update_consensus_for_slot(height, round);
                return HandleVoteResult::Success;
            }
            VoteType::Finalize(block_hash) => {
                let already_voted = validator_state.commit_state != CommitState::NoVoteYet;
                if already_voted {
                    return HandleVoteResult::ErrorValidatorHasAlreadyVotedInThisSlotRound;
                }

                validator_state.commit_state = CommitState::Finalize(block_hash);

                let block = self.get_block_candidate(height, round, &block_hash);
                block.finalize_votes.insert(validator_index, signature);
                block.finalize_stake += validator.stake;

                self.update_consensus_for_slot(height, round);
                return HandleVoteResult::Success;
            }
            VoteType::Skip => {
                let already_voted = validator_state.commit_state != CommitState::NoVoteYet;
                if already_voted {
                    return HandleVoteResult::ErrorValidatorHasAlreadyVotedInThisSlotRound;
                }

                validator_state.commit_state = CommitState::Skip;

                let round_state = self.round_state(height, round);
                round_state.skip_stake += validator.stake;
                round_state.skip_votes.insert(validator_index, signature);

                self.update_consensus_for_slot(height, round);
                return HandleVoteResult::Success;
            }
        }
    }
    fn handle_proposal_received(&mut self, proposal: Proposal) {
        let height = proposal.block.height;
        let round = proposal.round;
        if height < self.current_height {
            return;
        }
        let too_far_into_future = height >= self.current_height + MAX_SLOT_LOOKAHEAD;
        if too_far_into_future {
            return;
        }
        if round > self.current_round + MAX_ROUND_LOOKAHEAD {
            return;
        }
        let signed_by_leader = self.proposal_signed_by_leader(&proposal);
        if signed_by_leader {
            let state = self.round_state(height, round);

            let have_not_yet_received_block = state.leader_proposal.is_none();
            if have_not_yet_received_block {
                state.leader_proposal = Some(proposal.clone());
            }
            let block = proposal.block;
            let block_hash = block.calculate_hash();
            self.get_block_candidate(height, round, &block_hash).block = Some(block);
            self.update_consensus_for_slot(height, round);
        };
    }
    //double voting protection enforced through BTreeMap<ValidatorIndex,
    fn validate_cert_votes(
        &self,
        votes: &BTreeMap<ValidatorIndex, ed25519::Signature>,
        vote_type: VoteType,
        height: u64,
        round: u64,
    ) -> bool {
        if height >= self.current_height + MAX_SLOT_LOOKAHEAD {
            return false;
        }
        let Some(epoch_state) = self.epoch_state(height) else {
            return false;
        };
        let threshold = (epoch_state.total_validator_stake * 2) / 3;
        let mut valid_stake = 0;
        for (validator_index, signature) in votes {
            let Some(validator) = epoch_state.validator_data(*validator_index) else {
                continue;
            };
            let expected_vote = ValidatorVote {
                vote: vote_type.clone(),
                height,
                round,
                signature: *signature,
                validator_index: *validator_index,
            };
            if validator.pub_key.verify_sig(expected_vote.hash(), *signature) {
                valid_stake += validator.stake;
            }
        }
        let is_valid = valid_stake >= threshold;
        return is_valid;
    }
    fn handle_sync_finalized_block_received(&mut self, cert: FinalizedBlock) {
        let invalid_cert = !self.validate_cert_votes(
            &cert.votes,
            VoteType::Finalize(cert.block.calculate_hash()),
            cert.block.height,
            cert.round,
        );
        if invalid_cert {
            return;
        }
        let slot_state = self.slot_state(cert.block.height);
        slot_state.finalized_block = Some(cert);
    }
    fn handle_finalization_cert_received(&mut self, cert: FinalizationCertificate) {
        let invalid_cert = !self.validate_cert_votes(
            &cert.votes,
            VoteType::Finalize(cert.block_hash),
            cert.height,
            cert.round,
        );
        if invalid_cert {
            return;
        }
        let round_state = self.round_state(cert.height, cert.round);
        let block =
            round_state.block_candidates.get(&cert.block_hash).and_then(|c| c.block.clone());
        if let Some(block) = block {
            let block = FinalizedBlock { block, votes: cert.votes, round: cert.round };
            let slot_state = self.slot_state(cert.height);
            slot_state.finalized_block = Some(block);
        }
    }
    fn handle_justify_cert_received(&mut self, cert: JustifyCertificate) {
        let invalid_cert = !self.validate_cert_votes(
            &cert.votes,
            VoteType::Justify(cert.block_hash),
            cert.height,
            cert.round,
        );
        if invalid_cert {
            return;
        }
        let round_state = self.round_state(cert.height, cert.round);
        round_state.justify_cert = Some(cert);
    }
    fn handle_skip_cert_received(&mut self, cert: SkipCertificate) {
        let invalid_cert =
            !self.validate_cert_votes(&cert.votes, VoteType::Skip, cert.height, cert.round);
        if invalid_cert {
            return;
        }
        let round_state = self.round_state(cert.height, cert.round);
        round_state.skip_cert = Some(cert);
    }
    fn update_consensus_for_slot(&mut self, height: u64, round: u64) {
        let Some(epoch_state) = self.epoch_state(height) else { return };

        let total_validator_stake = epoch_state.total_validator_stake;
        let justification_threshold = (total_validator_stake * 2) / 3; //66%
        let finalization_threshold = (total_validator_stake * 2) / 3; //66%
        let skip_threshold = (total_validator_stake * 2) / 3; //66%

        let slot_state = self.slot_state(height);
        let round_state = slot_state.rounds.entry(round).or_default();

        let meets_skip_threshold = round_state.skip_stake >= skip_threshold;
        if meets_skip_threshold {
            let votes = round_state.skip_votes.clone();
            let cert = SkipCertificate { votes, round, height };
            round_state.skip_cert = Some(cert);
        }

        //check if any candidate can be justified or finalized
        for candidate in round_state.block_candidates.values() {
            let meets_justification_threshold = candidate.justify_stake >= justification_threshold;
            if meets_justification_threshold {
                let cert = JustifyCertificate {
                    block_hash: candidate.block_hash,
                    votes: candidate.justify_votes.clone(),
                    round,
                    height,
                };
                round_state.justify_cert = Some(cert);
            }

            let meets_finalization_threshold = candidate.finalize_stake >= finalization_threshold;
            let has_block = candidate.block.is_some();
            if meets_finalization_threshold && has_block {
                let block = FinalizedBlock {
                    block: candidate.block.clone().unwrap(),
                    votes: candidate.finalize_votes.clone(),
                    round,
                };
                slot_state.finalized_block = Some(block);
            }
        }
    }
    fn progress_consensus_round(&mut self) {
        let height = self.current_height;
        let state = self.slot_state.entry(height).or_default();

        //jump to latest justify cert if have newer justify cert
        if let Some(cert) = state.latest_justify_cert() {
            if cert.round >= self.current_round {
                let cert = cert.clone();
                //need to ensure votes on "jumped to justify cert round"
                self.current_round = cert.round;
                self.execute_validator_role();
                self.current_round = cert.round + 1;
                self.entered_round_at = Instant::now();
                self.sync_rounds_to_rpc();
                self.broadcast_justify_certificate(cert);
                self.persist_round_state();
                return;
            }
        }

        let skip_cert = self.round_state(height, self.current_round).skip_cert.clone();

        if let Some(cert) = skip_cert {
            self.current_round += 1;
            self.entered_round_at = Instant::now();
            self.sync_rounds_to_rpc();

            //https://docs.rs/commonware-consensus/latest/commonware_consensus/simplex/index.html#fetching-missing-certificates
            //While a more aggressive recovery mechanism could be employed, like requiring all participants to broadcast their highest finalization certificate after nullification
            self.broadcast_skip_certificate(cert);
            let latest_justify_cert = self.slot_state(height).latest_justify_cert().cloned();
            if let Some(latest_justify_cert) = latest_justify_cert {
                self.broadcast_justify_certificate(latest_justify_cert);
            }
            self.persist_round_state();
        }
    }
    fn progress_blockchain(&mut self) {
        let height = self.current_height;
        let slot = self.slot_state(height);

        let slot_is_finalized = slot.finalized_block.is_some();

        if slot_is_finalized {
            let finalized = slot.finalized_block.as_ref().unwrap().clone();
            self.broadcast_finalization_certificate(&finalized);
            self.latest_finalized_block = finalized.block.clone();

            self.execution.execute_block(finalized);

            self.entered_round_at = Instant::now();
            self.current_height += 1;
            self.current_round = 0;
            self.slot_state.retain(|&h, _| h >= self.current_height);
            self.clear_sync_rounds();
            self.clean_mempool();
        }
    }

    //https://docs.rs/commonware-consensus/latest/commonware_consensus/simplex/index.html#fetching-missing-certificates
    fn recover_sync(&mut self) {
        let stale = self.entered_round_at.elapsed() > Duration::from_secs(2);

        let vote_push_rate_limit = self.last_time_pushed_votes.elapsed() > Duration::from_secs(1);
        if stale && vote_push_rate_limit {
            //if stuck
            //  rebroadcast past round cert (round, finalization, null) every 5 seconds (r - 1)
            //  rebroadcast vote

            self.last_time_pushed_votes = Instant::now();
            self.push_votes();
            self.push_last_certificate();
        }

        let sync_rate_limit = self.last_sync_time.elapsed() > Duration::from_secs(1);
        if stale && sync_rate_limit {
            self.last_sync_time = Instant::now();
            self.sync_from_peer();
        }
    }
    fn push_votes(&mut self) {
        let height = self.current_height;
        let round = self.current_round;
        let validator_index = self.validator_index(self.pub_key, height).unwrap();

        let vote_state = self.local_validator_state(height, round);

        match vote_state.justify_state {
            JustifyState::NoVoteYet | JustifyState::Recovered => (),
            JustifyState::VotedFor(block_hash) => {
                let vote = ValidatorVote::create_signed(
                    VoteType::Justify(block_hash),
                    height,
                    round,
                    validator_index,
                    &self.private_key,
                );
                self.networking.broadcast_vote(&vote);
            }
        }

        let vote_state = self.local_validator_state(height, round);
        match vote_state.commit_state {
            CommitState::Finalize(block_hash) => {
                let vote = ValidatorVote::create_signed(
                    VoteType::Finalize(block_hash),
                    height,
                    round,
                    validator_index,
                    &self.private_key,
                );
                self.networking.broadcast_vote(&vote);
            }
            CommitState::Skip => {
                let vote = ValidatorVote::create_signed(
                    VoteType::Skip,
                    height,
                    round,
                    validator_index,
                    &self.private_key,
                );
                self.networking.broadcast_vote(&vote);
            }
            CommitState::NoVoteYet | CommitState::Recovered => (),
        };
    }
    fn push_last_certificate(&mut self) {
        let skip_cert = self.slot_state(self.current_height).latest_skip_cert().cloned();
        if let Some(cert) = skip_cert {
            self.broadcast_skip_certificate(cert);
        }

        let justify_cert = self.slot_state(self.current_height).latest_justify_cert().cloned();
        if let Some(cert) = justify_cert {
            self.broadcast_justify_certificate(cert);
        }
    }
    fn broadcast_justify_certificate(&self, cert: JustifyCertificate) {
        self.networking.broadcast_certificate(Certificate::Justify(cert));
    }
    fn broadcast_finalization_certificate(&self, finalized: &FinalizedBlock) {
        let cert = FinalizationCertificate {
            block_hash: finalized.block.calculate_hash(),
            height: finalized.block.height,
            votes: finalized.votes.clone(),
            round: finalized.round,
        };
        self.networking.broadcast_certificate(Certificate::Finalization(cert));
    }
    fn broadcast_skip_certificate(&self, cert: SkipCertificate) {
        self.networking.broadcast_certificate(Certificate::Skip(cert));
    }

    fn sync_from_peer(&mut self) {
        self.sync_blockchain();
        self.sync_rounds();
    }
    fn sync_blockchain(&mut self) {
        //sync from current to current +500
        //check which ones already have locally
        //and take(30)
        let start = self.current_height;
        let end = start + 500;

        let mut missing_heights = vec![];
        for height in start..end {
            if missing_heights.len() > 30 {
                //only request 30 per sync batch
                break;
            }
            let has_block =
                self.slot_state.get(&height).and_then(|s| s.finalized_block.as_ref()).is_some();
            if !has_block {
                missing_heights.push(height);
            }
        }
        for height in missing_heights {
            let networking = self.networking.clone();
            let block_sync_tx = self.block_sync_tx.clone();
            tokio::spawn(async move {
                let Some(reply) = networking.get_slot(height).await else {
                    return;
                };
                let Some(cert) = reply.slot else {
                    return;
                };
                let _ = block_sync_tx.send(cert);
            });
        }
    }
    fn sync_rounds(&mut self) {
        let height = self.current_height;
        let highest_skip_round = self.slot_state(height).highest_skip_round();

        //from current_round (current local tip) to latest observed certified tip (current chain tip)
        //fetch 5 at a time
        let missing_skips = self.current_round..highest_skip_round;

        for round in missing_skips.take(5) {
            let networking = self.networking.clone();
            let cert_tx = self.cert_tx.clone();
            tokio::spawn(async move {
                let Some(reply) = networking.get_round(height, round).await else {
                    return;
                };
                let Some(cert) = reply.cert else {
                    return;
                };

                let _ = cert_tx.send(cert);
            });
        }
    }
    fn sync_rounds_to_rpc(&mut self) {
        let current_round_for_sync = self.current_round_for_sync.clone();
        let state = self.slot_state.entry(self.current_height).or_default();
        let latest_justify_cert = state.latest_justify_cert().cloned();

        let skip_range = state.highest_justify_round()..state.highest_skip_round();
        let mut all_skips = HashMap::new();
        for round in skip_range {
            let skip_cert = self.round_state(self.current_height, round).skip_cert.clone();
            if let Some(skip_cert) = skip_cert {
                all_skips.insert(round, skip_cert);
            }
        }
        tokio::spawn(async move {
            let mut lock = current_round_for_sync.write().await;
            lock.latest_justify = latest_justify_cert.clone();
            lock.skip_certs = all_skips;
        });
    }
    fn clear_sync_rounds(&self) {
        let current_round_for_sync = self.current_round_for_sync.clone();
        tokio::spawn(async move {
            let mut lock = current_round_for_sync.write().await;
            lock.latest_justify = None;
            lock.skip_certs = HashMap::new();
        });
    }
    fn persist_round_state(&self) {
        let height = self.current_height;
        let Some(slot) = self.slot_state.get(&height) else { return };

        let mut skip_certs = Vec::new();
        for rs in slot.rounds.values() {
            if let Some(cert) = &rs.skip_cert {
                skip_certs.push(cert.clone());
            }
        }

        let latest_justify_cert = slot.latest_justify_cert().cloned();
        let mut justified_block = None;
        if let Some(cert) = &latest_justify_cert {
            if let Some(rs) = slot.rounds.get(&cert.round) {
                if let Some(bc) = rs.block_candidates.get(&cert.block_hash) {
                    justified_block = bc.block.clone();
                }
            }
        }

        self.db.write_round_state(&PersistedRoundState {
            height,
            current_round: self.current_round,
            skip_certs,
            latest_justify_cert,
            justified_block,
        });
    }
    fn local_validator_state(&mut self, height: u64, round: u64) -> &mut LocalValidatorState {
        //check if we have written to disk a vote larger then this height and round
        //if so to avoid double voting ensure no vote for this slot and round by setting state to votestate::recovered

        //will never vote on slot lower then most recently recovered height
        if height < self.last_disk_justify_vote.height {
            let state = self.round_state(height, round);
            state.local_validator_state.justify_state = JustifyState::Recovered;
        }
        //if same slot but lower round also dont
        if height == self.last_disk_justify_vote.height && round < self.last_disk_justify_vote.round
        {
            let state = self.round_state(height, round);
            state.local_validator_state.justify_state = JustifyState::Recovered;
        }

        //will never vote on slot lower then most recently recovered height
        if height < self.last_disk_commit_vote.height {
            let state = self.round_state(height, round);
            state.local_validator_state.commit_state = CommitState::Recovered;
        }
        //if same slot but lower round also dont
        if height == self.last_disk_commit_vote.height && round < self.last_disk_commit_vote.round {
            let state = self.round_state(height, round);
            state.local_validator_state.commit_state = CommitState::Recovered;
        }

        let state = self.round_state(height, round);
        return &mut state.local_validator_state;
    }
    fn leader(&self, height: u64, round: u64) -> Option<ValidatorData> {
        //not secure
        //alts,
        //actual vrf
        //vulnerable vrf, just do blockhash
        //queue based
        //will probs do vrf in future, just do vulnerable "random" for now?
        let epoch_state = self.epoch_state(height);

        //leader selection assume index
        if let Some(epoch_state) = epoch_state {
            let amount_of_validators = epoch_state.validator_data.len() as u64;
            let chosen_validator_index = (height + round) % amount_of_validators;

            let leader_pub_key =
                epoch_state.validator_index_to_key.get(&chosen_validator_index).unwrap();

            let leader = epoch_state.validator_data.get(leader_pub_key);
            return Some(*leader.unwrap());
        }
        return None;
    }
    fn is_leader(&self, public_key: ed25519::PublicKey, height: u64, round: u64) -> bool {
        if let Some(leader) = self.leader(height, round) {
            return leader.pub_key == public_key;
        }
        return false;
    }
    fn has_stake_at_height(&self, validator_public_key: ed25519::PublicKey, height: u64) -> bool {
        let Some(epoch_state) = self.epoch_state(height) else {
            return false;
        };
        let has_stake = epoch_state.validator_data.contains_key(&validator_public_key);
        return has_stake;
    }
    fn get_proposal(&mut self, height: u64, round: u64) -> Option<&Proposal> {
        let round_state = self.round_state(height, round);
        return round_state.leader_proposal.as_ref();
    }
    fn get_block_to_propose(&mut self) -> Option<Proposal> {
        let round = self.current_round;
        let height = self.current_height;

        let slot_state = self.slot_state(height);
        let has_justify_cert = slot_state.latest_justify_cert().is_some();

        if has_justify_cert {
            let justify_cert = slot_state.latest_justify_cert().unwrap();
            let justified_round = justify_cert.round;
            let block_hash = justify_cert.block_hash;

            let state = self.round_state(height, justified_round);
            let Some(candidate) = state.block_candidates.get(&block_hash) else {
                return None;
            };

            let Some(block) = &candidate.block else {
                return None;
            };

            let proposal = Proposal::create_signed(
                round,
                ProposalType::Reproposal(justified_round),
                block.clone(),
                &self.private_key,
            );
            return Some(proposal);
        } else {
            //propose new block
            let previous_block_hash = self.latest_finalized_block.calculate_hash();
            let mut transactions = Vec::new();
            let mut total_size = 0;
            for tx in self.mempool.values() {
                if transactions.len() >= MAX_TRANSACTIONS_PER_BLOCK {
                    break;
                }
                if !self.execution.verify_pow(tx) {
                    continue;
                }
                let tx_size = tx.encode().len();
                if total_size + tx_size > MAX_BLOCK_SIZE {
                    continue;
                }
                total_size += tx_size;
                transactions.push(tx.clone());
            }

            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

            let previous_block_state_root = self.execution.latest_state_root();
            let block = Block {
                height,
                transactions,
                previous_block_hash,
                timestamp,
                previous_block_state_root,
            };

            let proposal =
                Proposal::create_signed(round, ProposalType::Proposal, block, &self.private_key);

            return Some(proposal);
        }
    }
    fn get_valid_proposal(&mut self, height: u64, round: u64) -> Option<Proposal> {
        let Some(proposal) = self.get_proposal(height, round).cloned() else {
            return None;
        };
        let is_current_round = proposal.round == self.current_round;
        if !is_current_round {
            return None;
        }

        let mut block_size = 0;
        for tx in &proposal.block.transactions {
            block_size += borsh::to_vec(tx).unwrap().len();
        }
        if block_size > MAX_BLOCK_SIZE {
            return None;
        }

        match proposal.proposal_type {
            ProposalType::Proposal => {
                //for proposal of new block need skip certs for all rounds to current round
                let need_skip_certs_for_range = 0..round;
                for round in need_skip_certs_for_range {
                    let state = self.round_state(height, round);
                    let round_does_not_have_skip_cert = state.skip_cert.is_none();
                    if round_does_not_have_skip_cert {
                        return None;
                    }
                }

                let block = &proposal.block;
                let prev_block = &self.latest_finalized_block;

                let builds_on_latest = block.previous_block_hash == prev_block.calculate_hash();
                let timestamp_increased_or_same = block.timestamp >= prev_block.timestamp;
                let local_timestamp =
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                let timestamp_reasonable = block.timestamp < local_timestamp + 20;

                let is_current_height = block.height == self.current_height;
                let state_root_matches =
                    block.previous_block_state_root == self.execution.latest_state_root();

                let is_valid = builds_on_latest
                    && timestamp_increased_or_same
                    && is_current_height
                    && timestamp_reasonable
                    && state_root_matches;

                if is_valid {
                    return Some(proposal);
                } else {
                    return None;
                }
            }
            ProposalType::Reproposal(reproposed_round) => {
                //for reproposal needs skipcerts from reproposed_round to current round
                let need_skip_certs_for_range = (reproposed_round + 1)..round;
                for round in need_skip_certs_for_range {
                    let state = self.round_state(height, round);
                    let round_does_not_have_skip_cert = state.skip_cert.is_none();
                    if round_does_not_have_skip_cert {
                        return None;
                    }
                }
                //also need justify cert for reproposed_round that matches block hash of current
                let reproposed_state = self.round_state(height, reproposed_round);
                let Some(justify_cert) = reproposed_state.justify_cert.as_ref() else {
                    return None;
                };
                let justified_block_hash = justify_cert.block_hash;
                let reproposed_block_hash = proposal.block.calculate_hash();

                let hash_matches = justified_block_hash == reproposed_block_hash;

                let is_valid = hash_matches;

                if is_valid {
                    return Some(proposal);
                } else {
                    return None;
                }
            }
        }
    }
    fn proposal_signed_by_leader(&self, proposal: &Proposal) -> bool {
        let height = proposal.block.height;
        let round = proposal.round;
        let Some(leader) = self.leader(height, round) else {
            return false;
        };
        let hash = proposal.calculate_hash();
        let signed_by_leader = leader.pub_key.verify_sig(hash, proposal.leader_signature);

        return signed_by_leader;
    }

    fn validator_index(&self, public_key: ed25519::PublicKey, height: u64) -> Option<u64> {
        let epoch_state = self.epoch_state(height);
        if let Some(epoch_state) = epoch_state {
            if let Some(val) = epoch_state.validator_data.get(&public_key) {
                return Some(val.validator_index);
            }
            return None;
        }
        return None;
    }

    fn process_mempool(&mut self) {
        while let Ok(transaction) = self.transactions_rx.try_recv() {
            self.handle_new_mempool_tx(transaction);
        }
    }
    //assumes signature verification handled by networking.rs ingress
    fn handle_new_mempool_tx(&mut self, transaction: Transaction) {
        let tx_size = transaction.encode().len();
        if self.mempool_size + tx_size > MAX_MEMPOOL_SIZE {
            return;
        }
        let is_valid_mempool_tx = self.valid_mempool_tx(&transaction);
        if is_valid_mempool_tx {
            self.mempool_size += tx_size;
            self.mempool.insert(transaction.calculate_txhash(), transaction);
        }
    }
    fn clean_mempool(&mut self) {
        let mut tx_hash_to_remove = vec![];
        for transaction in self.mempool.values() {
            if !self.execution.verify_pow(transaction) {
                tx_hash_to_remove.push(transaction.calculate_txhash());
            }
        }
        for hash in tx_hash_to_remove {
            if let Some(tx) = self.mempool.remove(&hash) {
                self.mempool_size -= tx.encode().len();
            }
        }
    }

    fn valid_mempool_tx(&self, transaction: &Transaction) -> bool {
        self.execution.verify_pow(transaction)
    }

    fn epoch_state(&self, _height: u64) -> Option<&EpochState> {
        return self.epoch_states.first();
    }
    fn validator_state(
        &mut self,
        height: u64,
        round: u64,
        validator_index: u64,
    ) -> &mut ValidatorState {
        let default = ValidatorState {
            justify_state: JustifyState::NoVoteYet,
            commit_state: CommitState::NoVoteYet,
        };
        let round_state = self.round_state(height, round);
        let validator_state =
            round_state.validator_states.entry(validator_index).or_insert(default);
        return validator_state;
    }

    fn slot_state(&mut self, height: u64) -> &mut SlotState {
        let state = self.slot_state.entry(height).or_default();
        return state;
    }
    fn round_state(&mut self, height: u64, round: u64) -> &mut RoundState {
        let slot_state = self.slot_state.entry(height).or_default();
        let round_state = slot_state.rounds.entry(round).or_default();
        return round_state;
    }

    fn get_block_candidate(
        &mut self,
        height: u64,
        round: u64,
        block_hash: &Sha256Digest,
    ) -> &mut BlockCandidate {
        let state = self.round_state(height, round);
        let block_candidate = state.block_candidates.entry(*block_hash).or_insert(BlockCandidate {
            block_hash: *block_hash,
            block: None,
            justify_votes: BTreeMap::new(),
            justify_stake: 0,
            finalize_votes: BTreeMap::new(),
            finalize_stake: 0,
        });
        return block_candidate;
    }
    fn genesis_block() -> Block {
        let transactions = vec![];
        let height = 0;
        let previous_block_hash = Sha256Digest::from(*b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let timestamp = 0;
        let previous_block_state_root = Sha256Digest::default();
        let block = Block {
            height,
            transactions,
            previous_block_hash,
            timestamp,
            previous_block_state_root,
        };
        return block;
    }

    fn genesis_state(db: &Arc<Db>) -> InitialState {
        InitialState { block: Self::genesis_block(), execution: Execution::new(db.clone()) }
    }

    fn restore_from_db(db: &Arc<Db>) -> InitialState {
        let height = db.read_latest_finalized_height();
        let finalized = db.read_block(height).unwrap();
        InitialState { block: finalized.block, execution: Execution::restore_from_disk(db.clone()) }
    }

    fn recover_consensus_state(
        db: &Db,
        current_height: u64,
    ) -> (u64, HashMap<SlotHeight, SlotState>, LatestJustifyVote, LatestCommitVote) {
        let mut round = 0u64;
        let mut slot_state: HashMap<SlotHeight, SlotState> = HashMap::new();

        // Round state recovery
        if let Some(persisted) = db.read_round_state() {
            if persisted.height == current_height {
                round = persisted.current_round;

                let slot = slot_state.entry(current_height).or_default();

                for cert in persisted.skip_certs {
                    let rs = slot.rounds.entry(cert.round).or_default();
                    rs.skip_cert = Some(cert);
                }

                if let Some(cert) = persisted.latest_justify_cert {
                    let rs = slot.rounds.entry(cert.round).or_default();
                    rs.justify_cert = Some(cert.clone());

                    if let Some(block) = persisted.justified_block {
                        let block_hash = block.calculate_hash();
                        let bc = rs.block_candidates.entry(block_hash).or_insert(BlockCandidate {
                            block_hash,
                            block: None,
                            justify_votes: BTreeMap::new(),
                            justify_stake: 0,
                            finalize_votes: BTreeMap::new(),
                            finalize_stake: 0,
                        });
                        bc.block = Some(block);
                    }
                }
            }
        }

        // Justify vote recovery
        let last_justify_vote = db.read_last_justify_vote();
        match last_justify_vote.state.clone() {
            LatestJustifyVoteState::Justify(vote) => {
                let block_hash = vote.block.calculate_hash();

                let slot = slot_state.entry(last_justify_vote.height).or_default();
                let round_state = slot.rounds.entry(last_justify_vote.round).or_default();
                round_state.local_validator_state.justify_state =
                    JustifyState::VotedFor(block_hash);

                round_state.block_candidates.entry(block_hash).or_insert(BlockCandidate {
                    block_hash,
                    block: Some(vote.block),
                    justify_votes: BTreeMap::new(),
                    justify_stake: 0,
                    finalize_votes: BTreeMap::new(),
                    finalize_stake: 0,
                });
            }
            LatestJustifyVoteState::NoneYet => {}
        }

        // Commit vote recovery
        let last_commit_vote = db.read_last_commit_vote();
        match last_commit_vote.state.clone() {
            LatestCommitVoteState::Finalize(vote) => {
                let block_hash = vote.block_hash;

                let slot = slot_state.entry(last_commit_vote.height).or_default();
                let round_state = slot.rounds.entry(last_commit_vote.round).or_default();
                round_state.local_validator_state.commit_state = CommitState::Finalize(block_hash);

                round_state.block_candidates.entry(block_hash).or_insert(BlockCandidate {
                    block_hash,
                    block: None,
                    justify_votes: BTreeMap::new(),
                    justify_stake: 0,
                    finalize_votes: BTreeMap::new(),
                    finalize_stake: 0,
                });
            }
            LatestCommitVoteState::Skip => {
                let slot = slot_state.entry(last_commit_vote.height).or_default();
                let round_state = slot.rounds.entry(last_commit_vote.round).or_default();
                round_state.local_validator_state.commit_state = CommitState::Skip;
            }
            LatestCommitVoteState::NoneYet => {}
        }

        (round, slot_state, last_justify_vote, last_commit_vote)
    }

    async fn initialize(db: Arc<Db>, config: NodeConfig) -> Self {
        let (vote_tx, vote_rx) = mpsc::unbounded_channel::<ValidatorVote>();
        let (proposal_tx, proposal_rx) = mpsc::unbounded_channel::<Proposal>();
        let (transaction_tx, transactions_rx) = mpsc::unbounded_channel::<Transaction>();
        let (block_sync_tx, block_sync_rx) = mpsc::unbounded_channel::<FinalizedBlock>();
        let (cert_tx, cert_rx) = mpsc::unbounded_channel::<Certificate>();

        let current_round_for_sync = Arc::new(RwLock::new(RoundSyncStateExternal::default()));

        let private_key = config.keystore.validator_private_key.clone();
        let p2p_key = config.keystore.p2p_key.clone();
        let dtls_key = config.keystore.dtls_key;

        let networking = Networking::start(
            vote_tx.clone(),
            proposal_tx.clone(),
            cert_tx.clone(),
            transaction_tx,
            p2p_key,
            config.peers,
            current_round_for_sync.clone(),
            db.clone(),
            &config.genesis_epoch_state,
        )
        .await;

        if config.run_rpc_node {
            start_rpc_node(
                db.clone(),
                networking.clone(),
                dtls_key,
                config.rpc_nodes,
                config.genesis_epoch_state.clone(),
            );
        }

        //not optimal recovery logic
        let mut initial_state = Self::genesis_state(&db);
        let is_restart = db.read_latest_finalized_height() != 0;
        if is_restart {
            initial_state = Self::restore_from_db(&db);
        }
        let current_height = initial_state.block.height + 1;
        let (restored_round, restored_slot_state, last_disk_justify_vote, last_disk_commit_vote) =
            Self::recover_consensus_state(&db, current_height);

        ValidatorStateMachine {
            last_sync_time: Instant::now(),
            last_time_pushed_votes: Instant::now(),
            current_height,
            latest_finalized_block: initial_state.block,
            current_round: restored_round,
            epoch_states: vec![config.genesis_epoch_state],
            pub_key: private_key.public_key(),
            private_key,
            mempool: BTreeMap::new(),
            mempool_size: 0,
            networking,
            vote_rx,
            proposal_rx,
            cert_rx,
            cert_tx,
            execution: initial_state.execution,
            transactions_rx,
            slot_state: restored_slot_state,
            entered_round_at: Instant::now(),
            block_sync_rx,
            block_sync_tx,
            current_round_for_sync,
            last_disk_commit_vote,
            last_disk_justify_vote,
            db,
        }
    }
}

struct InitialState {
    block: Block,
    execution: Execution,
}

pub struct ValidatorStateMachine {
    current_round: u64,
    current_height: u64,
    latest_finalized_block: Block,
    entered_round_at: Instant,
    slot_state: HashMap<SlotHeight, SlotState>,

    last_sync_time: Instant,
    last_time_pushed_votes: Instant,

    last_disk_commit_vote: LatestCommitVote,
    last_disk_justify_vote: LatestJustifyVote,

    pub_key: ed25519::PublicKey,
    private_key: ed25519::PrivateKey,

    networking: Arc<Networking>,
    execution: Execution,
    db: Arc<Db>,

    epoch_states: Vec<EpochState>,

    vote_rx: UnboundedReceiver<ValidatorVote>,
    proposal_rx: UnboundedReceiver<Proposal>,
    block_sync_rx: UnboundedReceiver<FinalizedBlock>,
    cert_rx: UnboundedReceiver<Certificate>,

    block_sync_tx: UnboundedSender<FinalizedBlock>,
    cert_tx: UnboundedSender<Certificate>,

    current_round_for_sync: Arc<RwLock<RoundSyncStateExternal>>,

    mempool: BTreeMap<Sha256Digest, Transaction>,
    mempool_size: usize,
    transactions_rx: UnboundedReceiver<Transaction>,
}

#[derive(Clone)]
pub struct EpochState {
    pub validator_index_to_key: BTreeMap<u64, ed25519::PublicKey>,
    pub validator_data: BTreeMap<ed25519::PublicKey, ValidatorData>,
    pub total_validator_stake: u64,
}
impl EpochState {
    pub fn new() -> EpochState {
        return EpochState {
            validator_index_to_key: BTreeMap::new(),
            validator_data: BTreeMap::new(),
            total_validator_stake: 0,
        };
    }
    pub fn validator_data(&self, validator_index: u64) -> Option<&ValidatorData> {
        let Some(validator_public_key) = self.validator_index_to_key.get(&validator_index) else {
            return None;
        };
        let validator_data = self.validator_data.get(validator_public_key).expect("invariant");
        return Some(validator_data);
    }
    pub fn add_registered_validator(
        &mut self,
        pub_key: ed25519::PublicKey,
        p2p_key: ed25519::PublicKey,
        stake: u64,
    ) {
        let current_length = self.validator_data.len();
        let validator_index = current_length as u64;
        self.validator_data
            .insert(pub_key, ValidatorData { stake, validator_index, pub_key, p2p_key });
        self.validator_index_to_key.insert(validator_index, pub_key);
        self.total_validator_stake += stake;
    }
}

pub type SlotHeight = u64;
pub type ValidatorIndex = u64;

#[derive(Debug, Clone)]
pub struct BlockCandidate {
    pub block_hash: Sha256Digest,
    pub block: Option<Block>,

    pub justify_votes: BTreeMap<ValidatorIndex, ed25519::Signature>,
    pub justify_stake: u64,

    pub finalize_votes: BTreeMap<ValidatorIndex, ed25519::Signature>,
    pub finalize_stake: u64,
}

#[derive(Debug)]
pub struct RoundState {
    pub block_candidates: BTreeMap<Sha256Digest, BlockCandidate>,
    pub skip_stake: u64,
    pub skip_votes: BTreeMap<ValidatorIndex, ed25519::Signature>,

    pub validator_states: HashMap<ValidatorIndex, ValidatorState>,
    pub leader_proposal: Option<Proposal>,
    pub local_validator_state: LocalValidatorState,

    pub skip_cert: Option<SkipCertificate>,
    pub justify_cert: Option<JustifyCertificate>,
}
impl Default for RoundState {
    fn default() -> RoundState {
        return RoundState {
            justify_cert: None,
            skip_cert: None,
            block_candidates: BTreeMap::new(),
            skip_stake: 0,
            skip_votes: BTreeMap::new(),
            validator_states: HashMap::new(),
            leader_proposal: None,
            local_validator_state: LocalValidatorState {
                justify_state: JustifyState::NoVoteYet,
                commit_state: CommitState::NoVoteYet,
                leader_state: LeaderState::HasNotProposedBlock,
            },
        };
    }
}
#[derive(Debug, Default)]
pub struct SlotState {
    pub finalized_block: Option<FinalizedBlock>,
    pub rounds: HashMap<u64, RoundState>,
}

impl SlotState {
    fn latest_justify_cert(&self) -> Option<&JustifyCertificate> {
        let mut highest: Option<&JustifyCertificate> = None;
        for round_state in self.rounds.values() {
            let Some(cert) = &round_state.justify_cert else { continue };
            if highest.is_none() {
                highest = Some(cert);
            } else if cert.round > highest.unwrap().round {
                highest = Some(cert);
            }
        }
        return highest;
    }
    fn highest_justify_round(&self) -> u64 {
        match self.latest_justify_cert() {
            Some(cert) => cert.round,
            None => 0,
        }
    }
    fn latest_skip_cert(&self) -> Option<&SkipCertificate> {
        let mut highest: Option<&SkipCertificate> = None;
        for round_state in self.rounds.values() {
            let Some(cert) = &round_state.skip_cert else { continue };
            if highest.is_none() {
                highest = Some(cert);
            } else if cert.round > highest.unwrap().round {
                highest = Some(cert);
            }
        }
        return highest;
    }
    fn highest_skip_round(&self) -> u64 {
        match self.latest_skip_cert() {
            Some(cert) => cert.round,
            None => 0,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ValidatorData {
    pub stake: u64,
    pub validator_index: u64,
    pub pub_key: ed25519::PublicKey,
    pub p2p_key: ed25519::PublicKey,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HandleVoteResult {
    ErrorTooFarIntoFuture,
    ErrorInvalidSignature,
    ErrorEpochStateDoesNotYetExistForSlot,
    ErrorCouldNotFindValidatorIndex,
    ErrorValidatorHasAlreadyVotedInThisSlotRound,
    ErrorSlotAlreadyFinalized,
    Success,
}

#[derive(PartialEq, Debug)]
pub enum LeaderState {
    HasProposedBlock,
    HasNotProposedBlock,
}
#[derive(PartialEq, Debug)]
pub struct ValidatorState {
    justify_state: JustifyState,
    commit_state: CommitState,
}
#[derive(PartialEq, Debug)]
pub struct LocalValidatorState {
    justify_state: JustifyState,
    commit_state: CommitState,
    leader_state: LeaderState,
}
#[derive(PartialEq, Debug)]
pub enum JustifyState {
    NoVoteYet,
    VotedFor(Sha256Digest),
    Recovered,
}
#[derive(PartialEq, Debug)]
pub enum CommitState {
    NoVoteYet,
    Finalize(Sha256Digest),
    Skip,
    Recovered,
}
pub struct NodeConfig {
    pub keystore: Keystore,
    pub peers: Vec<KnownPeer>,
    pub run_rpc_node: bool,
    pub genesis_epoch_state: EpochState,
    pub rpc_nodes: Vec<vastrum_shared_types::frontend::frontend_data::RpcNodeEndpoint>,
}
use crate::utils::limits::{LONG_ROUND_TIMEOUT, ROUND_TIMEOUT};
use crate::{
    consensus::types::{
        Block, Certificate, FinalizationCertificate, FinalizedBlock, JustifyCertificate, Proposal,
        ProposalType, RoundSyncStateExternal, SkipCertificate, ValidatorVote, VoteType,
    },
    db::{
        Db,
        round_state::PersistedRoundState,
        vote_state::{
            LatestCommitVote, LatestCommitVoteState, LatestJustifyVote, LatestJustifyVoteState,
        },
    },
    execution::execution::Execution,
    keystore::keyset::Keystore,
    p2p::{networking::Networking, peer_manager::KnownPeer},
    rpc::start::start_rpc_node,
    utils::limits::{MAX_MEMPOOL_SIZE, MAX_ROUND_LOOKAHEAD, MAX_SLOT_LOOKAHEAD},
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    time::{Instant, sleep},
};
use vastrum_shared_types::{
    borsh::BorshExt,
    crypto::{ed25519, sha256::Sha256Digest},
    limits::{MAX_BLOCK_SIZE, MAX_TRANSACTIONS_PER_BLOCK},
    types::execution::transaction::Transaction,
};
