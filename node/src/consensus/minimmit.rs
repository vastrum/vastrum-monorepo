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
impl ValidatorStateMachine {
    pub async fn run(&mut self, addr: SocketAddr, peers: Vec<NodeRecord>, run_rpc_server: bool) {
        self.start_services(addr, peers, run_rpc_server).await;

        self.viewchain.insert(0, SlotState::Block(self.genesis_block()));
        self.epoch_states.push(self.genesis_epoch_state());

        loop {
            self.process_consensus_messages();

            self.validator_actor();

            self.progress_viewchain();
            self.progress_blockchain();

            self.process_mempool();

            self.debug_write_blockchain();
            self.sleep().await;
        }
    }
    fn validator_actor(&mut self) {
        let slot_height = self.latest_view_height + 1;
        let is_leader = self.is_leader_for_slot(&self.pub_key, slot_height);
        let is_validator = self.has_stake_in_current_epoch(&self.pub_key, slot_height);

        if is_leader {
            self.exec_leader_role();
        }

        if is_validator {
            self.exec_validator_role();
        }
    }
    fn exec_leader_role(&mut self) {
        let slot_height = self.latest_view_height + 1;
        let state = self.get_local_validator_state_for_slot(slot_height);

        let has_not_proposed_block = state.leader_state == LeaderState::HasNotProposedBlock;

        if has_not_proposed_block {
            state.leader_state = LeaderState::HasProposedBlock;
            let block_to_propose = self.get_block_to_propose(slot_height).expect("todo");
            self.networking.broadcast_block_proposal(&block_to_propose);
            self.handle_block_proposal_received(block_to_propose);
        }
    }
    fn exec_validator_role(&mut self) {
        let slot_height = self.latest_view_height + 1;
        let validator_index = self.get_validator_index(&self.pub_key, slot_height).unwrap();
        let validator_state = self.get_local_validator_state_for_slot(slot_height);

        let has_not_voted_yet = validator_state.vote_state == VoteState::NoVoteYet;
        let has_voted_for_block = matches!(validator_state.vote_state, VoteState::VotedForBlock(_));

        if has_not_voted_yet {
            let has_block_proposal = self.has_block_proposal_for_slot(slot_height);
            let deadline_expired = self.slot_start + Duration::from_secs(2) <= Instant::now();

            if deadline_expired {
                let current_validator_state = self.get_local_validator_state_for_slot(slot_height);
                current_validator_state.vote_state = VoteState::VotedForNullification;

                let null_hash = ValidatorStateMachine::calculate_null_hash(slot_height);
                let signature = self.private_key.sign_hash(null_hash);

                let vote = NullVote { slot_height, signature, validator_index };

                self.networking.broadcast_null_vote(&vote);
                self.handle_nullification_vote(vote);
            } else if has_block_proposal {
                let block_proposal = self.get_slot_block_proposal(slot_height).unwrap();
                let block_hash = block_proposal.calculate_block_hash();

                let current_validator_state = self.get_local_validator_state_for_slot(slot_height);
                current_validator_state.vote_state = VoteState::VotedForBlock(block_hash);

                let signature = self.private_key.sign_hash(block_hash);
                let vote = BlockVote { slot_height, block_hash, signature, validator_index };

                self.networking.broadcast_block_vote(&vote);
                self.handle_block_vote(vote);
            }
        }

        let slot_consensus_states = self.get_slot_consensus_result(slot_height);
        let slot_implicitly_nullified = slot_consensus_states == Consensus::ImplicitNullification;

        //no block can ever finalize in this slot, to avoid consensus halting change vote to null
        if has_voted_for_block && slot_implicitly_nullified {
            let nullification_hash = ValidatorStateMachine::calculate_null_hash(slot_height);
            let signature = self.private_key.sign_hash(nullification_hash);
            let vote = NullVote { slot_height, signature, validator_index };

            self.networking.broadcast_null_vote(&vote);
            self.handle_nullification_vote(vote);

            let current_validator_state = self.get_local_validator_state_for_slot(slot_height);
            current_validator_state.vote_state = VoteState::VotedForNullification;
        }
    }
    fn progress_viewchain(&mut self) {
        let height = self.latest_view_height + 1;
        self.debug_view_info(height);

        let viewchain_was_nullified = self.check_for_viewchain_nullification();
        if viewchain_was_nullified {
            return;
        }

        //To avoid double vote issues on forks when switching from (null, height: 10) > (block, height:11, parent: block:10)
        //if have voted on a slot will only change view in specific conditions
        //if voted none
        //  all
        //if voted block
        //  only views of same block and finalizations of any and nullifications
        //if voted null
        //  only finalizations and nullification

        let slot_consensus = self.get_slot_consensus_result(height);
        let validator_state = self.get_local_validator_state_for_slot(height);

        let has_not_voted_yet = validator_state.vote_state == VoteState::NoVoteYet;
        let has_voted_for_block = matches!(validator_state.vote_state, VoteState::VotedForBlock(_));
        let has_voted_null = validator_state.vote_state == VoteState::VotedForNullification;

        if has_not_voted_yet {
            if let Consensus::FinalizedNullification(notarization) = slot_consensus {
                self.add_nullification_to_viewchain(height, notarization);
            } else if let Consensus::ViewBlock(notarization) = slot_consensus {
                self.add_block_to_viewchain(height, notarization);
            } else if let Consensus::FinalizedBlock(notarization) = slot_consensus {
                self.add_block_to_viewchain(height, notarization);
            }
        } else if has_voted_for_block {
            let VoteState::VotedForBlock(block_voted_for) = validator_state.vote_state else {
                return;
            };

            if let Consensus::FinalizedNullification(notarization) = slot_consensus {
                self.add_nullification_to_viewchain(height, notarization);
            } else if let Consensus::ViewBlock(notarization) = slot_consensus {
                let block_is_block_voted_for = notarization.block_hash == block_voted_for;
                if block_is_block_voted_for {
                    self.add_block_to_viewchain(height, notarization);
                }
            } else if let Consensus::FinalizedBlock(notarization) = slot_consensus {
                self.add_block_to_viewchain(height, notarization);
            }
        } else if has_voted_null {
            if let Consensus::FinalizedNullification(notarization) = slot_consensus {
                self.add_nullification_to_viewchain(height, notarization);
            } else if let Consensus::FinalizedBlock(notarization) = slot_consensus {
                self.add_block_to_viewchain(height, notarization);
            }
        }
        self.execute_recovery_if_stale(height);
    }
    fn execute_recovery_if_stale(&mut self, height: u64) {
        let is_recovery = self.slot_start.elapsed() > Duration::from_secs(3);

        let pull_rate_limit = self.last_time_asked_for_votes.elapsed() > Duration::from_secs(2);
        if is_recovery && pull_rate_limit {
            self.last_time_asked_for_votes = Instant::now();
            self.download_100_notarization_from_peer(self.latest_non_cancellable_slot_height);
            self.download_100_notarization_from_peer(height);
        }

        let push_rate_limit = self.last_time_rebroadcast_votes.elapsed() > Duration::from_secs(2);
        if is_recovery && push_rate_limit {
            self.last_time_rebroadcast_votes = Instant::now();
            self.rebroadcast_all_votes(height);
        }
    }
    fn rebroadcast_all_votes(&mut self, slot_height: u64) {
        let slc =
            self.slot_consensus.entry(slot_height).or_insert(SlotConsensusState::new(slot_height));
        for candidate in &slc.block_candidates {
            let block_hash = *candidate.0;
            let state = candidate.1;
            for vote in state.votes.clone() {
                let validator_index = vote.0;
                let signature = vote.1;
                let block_vote = BlockVote { validator_index, signature, block_hash, slot_height };
                self.networking.broadcast_block_vote(&block_vote);
            }
        }
        for null_vote in &slc.nullification_votes {
            let validator_index = *null_vote.0;
            let signature = *null_vote.1;
            self.networking.broadcast_null_vote(&NullVote {
                slot_height,
                signature,
                validator_index,
            });
        }
    }
    fn add_block_to_viewchain(&mut self, height: u64, notarization: BlockNotarization) {
        let Some(block) = self.get_block(height, &notarization.block_hash) else {
            self.download_100_blocks_from_peer(height);
            return;
        };
        let valid_block = self.valid_block_to_add_to_viewchain(&block, height);
        if !valid_block {
            return;
        }
        let mut notarizations = vec![];
        for (val_index, sig) in notarization.votes {
            let vote = Notarization { validator_index: val_index, signature: sig };
            notarizations.push(vote);
        }

        let notarized_block = NotarizedBlock {
            height: block.height,
            transactions: block.transactions,
            previous_block_hash: block.previous_block_hash,
            slot_leader_signature: block.slot_leader_signature,
            votes: notarizations,
        };

        info!("New view block at height {}, {:#?}", height, notarized_block);
        self.viewchain.insert(height, SlotState::Block(notarized_block.clone()));
        self.latest_view_height += 1;

        self.database.write(SlotState::Block(notarized_block));
        self.slot_start = Instant::now();
        self.calculate_view_seen_pow_hash();
    }

    fn add_nullification_to_viewchain(
        &mut self,
        height: u64,
        notarization: NullificationNotarization,
    ) {
        let mut votes = vec![];
        for (validator_index, signature) in notarization.votes {
            let note = Notarization { validator_index, signature };
            votes.push(note);
        }

        let nullification = NotarizedNullification { height, votes };
        info!("New nullification at height {}, {:#?}", height, nullification);

        self.viewchain.insert(height, SlotState::Nullification(nullification.clone()));
        self.latest_view_height += 1;

        self.database.write(SlotState::Nullification(nullification));
        self.slot_start = Instant::now();
        self.calculate_view_seen_pow_hash();
    }
    fn check_for_viewchain_nullification(&mut self) -> bool {
        for height in (self.latest_non_cancellable_slot_height + 1)..=self.latest_view_height {
            let slot = self.slot_consensus.get(&height).unwrap();

            let view = self.viewchain.get(&height).unwrap();
            let current_view_is_block = matches!(view, SlotState::Block(_));

            //let implicit_null = slot.consensus == Consensus::ImplicitNullification;
            //let slot_is_nullified = implicit_null | nullified;

            let slot_nullified = matches!(slot.consensus, Consensus::FinalizedNullification(_));
            let view_has_been_nullified = current_view_is_block && slot_nullified;
            if view_has_been_nullified {
                if let Consensus::FinalizedNullification(notarization) = &slot.consensus {
                    info!(
                        "View at height {height} was nullified, all descendant views up to latest_view_height {} has also been nullified",
                        self.latest_view_height
                    );

                    //remove all blocks after this nullified slot height
                    let nulled_blocks = self.viewchain.split_off(&height);

                    self.latest_view_height = height - 1;
                    self.add_nullification_to_viewchain(
                        self.latest_view_height + 1,
                        notarization.clone(),
                    );
                    self.clean_seen_tx_viewchain_reorg(nulled_blocks);
                    return true;
                }
            }
        }
        return false;
    }
    fn valid_block_to_add_to_viewchain(&self, block: &Block, height: u64) -> bool {
        let valid_height = height == block.height;
        let BlockVerificationResult::Verifiable(valid_block) = self.verify_block(&block) else {
            info!("Could not verify view block, delaying, height is:  {}", height);
            return false;
        };
        let is_valid = valid_block && valid_height;
        if !is_valid {
            info!("Invalid view, delaying {}", height);
        }
        return is_valid;
    }
    fn calculate_view_seen_pow_hash(&mut self) {
        let mut new_seen_pow_hash = HashSet::new();
        for view in &self.viewchain {
            if let SlotState::Block(block) = view.1 {
                for transaction in &block.transactions {
                    let pow_hash = transaction.calculate_pow_hash();
                    new_seen_pow_hash.insert(pow_hash);
                }
            }
        }
        self.view_consumed_pow_hashes = new_seen_pow_hash;
    }
    fn check_consensus_for_slot(slc: &SlotConsensusState, total_validator_stake: u64) -> Consensus {
        let view_progression_threshold = (total_validator_stake * 2) / 5; //40%
        let block_finalization_threshold = (total_validator_stake * 4) / 5; //80%
        let nullification_threshold = (total_validator_stake * 2) / 5; //40%
        let implicit_nullification_threshold = (total_validator_stake * 2) / 5; //40%

        let meets_nullification_threshold = slc.null_stake >= nullification_threshold;
        if meets_nullification_threshold {
            let notarization = NullificationNotarization {
                votes: slc.nullification_votes.clone(),
                height: slc.slot_height,
            };
            return Consensus::FinalizedNullification(notarization);
        }

        //check if slot is implicitly nullified
        //will implicitly nullify this slot if
        //there is no possibility of any block reaching 80% stake while allowing for 20% double votes
        let mut highest_block_votes = 0;
        let mut total_block_votes = 0;
        for candidate in slc.block_candidates.values() {
            total_block_votes += candidate.voted_stake;
            if candidate.voted_stake > highest_block_votes {
                highest_block_votes = candidate.voted_stake;
            }
        }
        let non_leading_block_votes = total_block_votes - highest_block_votes;

        let null_votes = slc.null_stake;
        let implicitly_nullified =
            null_votes + non_leading_block_votes >= implicit_nullification_threshold;
        if implicitly_nullified {
            return Consensus::ImplicitNullification;
        }

        //finally check if slot can viewize or finalize to block
        for (block_hash, candidate) in &slc.block_candidates {
            let voted_stake = candidate.voted_stake;

            let block_meets_finalization_threshold = voted_stake >= block_finalization_threshold;
            let block_meets_view_threshold = voted_stake >= view_progression_threshold;

            let notarization = BlockNotarization {
                block_hash: block_hash.clone(),
                height: slc.slot_height,
                votes: candidate.votes.clone(),
            };

            if block_meets_finalization_threshold {
                return Consensus::FinalizedBlock(notarization);
            } else if block_meets_view_threshold {
                return Consensus::ViewBlock(notarization);
            }
        }

        return Consensus::None;
    }
    fn progress_blockchain(&mut self) {
        let height = self.latest_non_cancellable_slot_height + 1;
        info!("progress blockchain");
        if self.viewchain.get(&height).is_none() {
            return;
        }
        let Some(slot) = self.slot_consensus.get(&height) else {
            return;
        };

        let view_finalized = matches!(slot.consensus, Consensus::FinalizedBlock(_));
        let slot_nullified = matches!(slot.consensus, Consensus::FinalizedNullification(_));

        if view_finalized {
            let new_viewchain = self.viewchain.split_off(&height);
            let viewchain_to_write_to_db = std::mem::take(&mut self.viewchain);
            //let viewchain_to_write_to_db = self.viewchain.clone();
            self.viewchain = new_viewchain;

            //write everything before height to blockchain as now finalized
            //will only be nullified slots before latest height and 1 block at latest height
            for slot_state in viewchain_to_write_to_db.values() {
                let slot_consensus = self.get_slot_consensus_result(height);
                if let SlotState::Block(block) = slot_state {
                    if let Consensus::FinalizedBlock(notarization) = slot_consensus {
                        let mut notarizations = vec![];
                        for (val_index, sig) in notarization.votes {
                            let vote = Notarization { validator_index: val_index, signature: sig };
                            notarizations.push(vote);
                        }

                        let notarized_block = NotarizedBlock {
                            height: block.height,
                            transactions: block.transactions.clone(),
                            previous_block_hash: block.previous_block_hash,
                            slot_leader_signature: block.slot_leader_signature.clone(),
                            votes: notarizations,
                        };
                        self.execution.execute_block(notarized_block.clone());
                        self.database.write(SlotState::Block(notarized_block));
                    }
                } else if let SlotState::Nullification(_) = slot_state {
                    if let Consensus::FinalizedNullification(notarization) = slot_consensus {
                        let mut votes = vec![];
                        for (validator_index, signature) in notarization.votes {
                            let note = Notarization {
                                validator_index: validator_index,
                                signature: signature,
                            };
                            votes.push(note);
                        }

                        let null = NotarizedNullification { height: height, votes };
                        self.database.write(SlotState::Nullification(null));
                    }
                }
                self.slot_consensus.remove(&height);
            }
            self.latest_finalized_block_height = height;
            self.latest_non_cancellable_slot_height = height;
            self.calculate_view_seen_pow_hash();
            self.clean_mempool();
        } else if slot_nullified {
            self.latest_non_cancellable_slot_height += 1;
            self.calculate_view_seen_pow_hash();
        }
    }

    fn process_consensus_messages(&mut self) {
        while let Ok(block_vote) = self.block_vote_rx.try_recv() {
            self.handle_block_vote(block_vote);
        }
        while let Ok(nullification_vote) = self.nullification_vote_rx.try_recv() {
            self.handle_nullification_vote(nullification_vote);
        }
        while let Ok(block_proposal) = self.proposedblock_rx.try_recv() {
            self.handle_block_proposal_received(block_proposal);
        }
    }
    fn handle_block_vote(&mut self, block_vote: BlockVote) -> HandleBlockVoteResult {
        let slot_height = block_vote.slot_height;
        let signature = block_vote.signature;
        let block_hash = block_vote.block_hash;
        let validator_index = block_vote.validator_index;

        if slot_height <= self.latest_finalized_block_height {
            return HandleBlockVoteResult::ErrorSlotAlreadyFinalized;
        }

        let Some(epoch_state) = self.get_epoch_state_for_slot(slot_height) else {
            return HandleBlockVoteResult::ErrorEpochStateDoesNotYetExistForSlot;
        };

        let Some(validator_slot_state) = epoch_state.get_validator_data(validator_index) else {
            return HandleBlockVoteResult::ErrorCouldNotFindValidatorIndex;
        };
        let is_valid_signature =
            validator_slot_state.pub_key.verify_signature_hash(block_vote.block_hash, &signature);

        if !is_valid_signature {
            return HandleBlockVoteResult::ErrorInvalidSignature;
        }
        let validator_state = self.get_validator_slot_vote_state(slot_height, validator_index);

        let has_not_voted_yet = *validator_state == VoteState::NoVoteYet;
        if has_not_voted_yet {
            *validator_state = VoteState::VotedForBlock(block_hash.clone());

            let block_candidate = self.get_block_candidate_or_default(slot_height, &block_hash);
            block_candidate.votes.insert(validator_index, signature);
            block_candidate.voted_stake += validator_slot_state.stake;

            let slot_consensus_state = self.get_slot_consensus_or_default(slot_height);
            let consensus_state_result = ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                epoch_state.total_validator_stake,
            );
            slot_consensus_state.consensus = consensus_state_result.clone();

            return HandleBlockVoteResult::Success(consensus_state_result);
        } else {
            return HandleBlockVoteResult::ErrorValidatorHasAlreadyVotedInThisSlot;
        }
    }
    fn handle_nullification_vote(&mut self, null_vote: NullVote) -> HandleNullificationVoteResult {
        let slot_height = null_vote.slot_height;
        let signature = null_vote.signature;
        let validator_index = null_vote.validator_index;

        if slot_height <= self.latest_finalized_block_height {
            return HandleNullificationVoteResult::ErrorSlotAlreadyFinalized;
        }
        let Some(epoch_state) = self.get_epoch_state_for_slot(slot_height) else {
            return HandleNullificationVoteResult::ErrorEpochStateDoesNotYetExistForSlot;
        };
        let Some(validator_slot_state) = epoch_state.get_validator_data(validator_index) else {
            return HandleNullificationVoteResult::ErrorCouldNotFindValidatorIndex;
        };

        let nullification_hash = ValidatorStateMachine::calculate_null_hash(slot_height);
        let is_valid_signature =
            &validator_slot_state.pub_key.verify_signature_hash(nullification_hash, &signature);

        if !is_valid_signature {
            return HandleNullificationVoteResult::ErrorInvalidSignature;
        }

        //If validator previously voted for a block, remove that vote
        let validator_vote_state = self.get_validator_slot_vote_state(slot_height, validator_index);

        if let VoteState::VotedForBlock(previously_voted_for_block_hash) = validator_vote_state {
            let hash = previously_voted_for_block_hash.clone();

            let candidate = self.get_block_candidate_or_default(slot_height, &hash);
            candidate.voted_stake -= validator_slot_state.stake;
            candidate.votes.remove(&validator_index);
        }

        let validator_vote_state = self.get_validator_slot_vote_state(slot_height, validator_index);

        let voted_for_block = matches!(validator_vote_state, VoteState::VotedForBlock(_));
        let no_vote_yet = *validator_vote_state == VoteState::NoVoteYet;

        let has_not_previously_voted_null = voted_for_block || no_vote_yet;

        if has_not_previously_voted_null {
            *validator_vote_state = VoteState::VotedForNullification;

            let slot_consensus_state = self.get_slot_consensus_or_default(slot_height);
            slot_consensus_state
                .consumed_validator_indexes
                .insert(validator_index, VoteState::VotedForNullification);
            slot_consensus_state.null_stake += validator_slot_state.stake;
            slot_consensus_state.nullification_votes.insert(validator_index, signature);

            let consensus_state_result = ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                epoch_state.total_validator_stake,
            );
            slot_consensus_state.consensus = consensus_state_result.clone();
            return HandleNullificationVoteResult::Success(consensus_state_result);
        } else {
            return HandleNullificationVoteResult::ErrorValidatorHasAlreadyVotedNullInThisSlot;
        }
    }
    fn handle_block_proposal_received(&mut self, block: Block) {
        if block.height <= self.latest_finalized_block_height {
            return;
        }

        let block_verification = self.verify_block(&block);

        let could_not_verify_block =
            block_verification == BlockVerificationResult::CouldNotVerifyDefer;
        //if could not verify block, then store it for future verification
        //TODO dos vector, fix this
        if could_not_verify_block {
            self.slot_consensus
                .entry(block.height)
                .or_insert(SlotConsensusState::new(block.height))
                .unverified_blocks_received
                .insert(block.calculate_block_hash(), block);
        } else if let BlockVerificationResult::Verifiable(correct_block) = block_verification {
            //TODO: should not overwrite, should only write once and then and further writes should not be added
            //incase already exists a block
            //so proposer double propose blocks for a local node
            if correct_block {
                self.slot_consensus
                    .entry(block.height)
                    .or_insert(SlotConsensusState::new(block.height))
                    .first_leader_signed_block_received = Some(block.clone());
            }
        };
    }

    fn process_mempool(&mut self) {
        while let Ok(transaction) = self.transactions_rx.try_recv() {
            println!("new transaction received to mempool {:#?}", transaction);
            self.handle_new_mempool_tx(transaction);
        }
    }
    fn handle_new_mempool_tx(&mut self, transaction: Transaction) {
        let is_valid_mempool_tx = self.valid_mempool_tx(&transaction);
        println!("new mempool tx isvalid={}", is_valid_mempool_tx);
        if is_valid_mempool_tx {
            self.mempool.insert(transaction.calculate_txhash(), transaction);
        }
    }
    fn clean_mempool(&mut self) {
        let mut tx_hash_to_remove = vec![];
        let mut pow_hash_to_remove = vec![];

        for transaction in self.mempool.values() {
            let invalid_tx = !self.execution.verify_transaction(&transaction);
            if invalid_tx {
                tx_hash_to_remove.push(transaction.calculate_txhash());
                pow_hash_to_remove.push(transaction.calculate_pow_hash());
            }
        }
        for hash in tx_hash_to_remove {
            self.mempool.remove(&hash);
            self.view_consumed_txs.remove(&hash);
        }
        for hash in pow_hash_to_remove {
            self.view_consumed_pow_hashes.remove(&hash);
        }
    }
    fn clean_seen_tx_viewchain_reorg(&mut self, blocks: BTreeMap<u64, SlotState>) {
        for block in blocks.values() {
            let SlotState::Block(block) = block else {
                continue;
            };
            for transaction in &block.transactions {
                let invalid_tx = !self.execution.verify_transaction(&transaction);
                if invalid_tx {
                    self.view_consumed_txs.remove(&transaction.calculate_txhash());
                    self.view_consumed_pow_hashes.remove(&transaction.calculate_pow_hash());
                }
            }
        }
    }
    fn valid_mempool_tx(&self, transaction: &Transaction) -> bool {
        let pow_hash = transaction.calculate_pow_hash();

        let pow_not_yet_used_in_view = !self.view_consumed_pow_hashes.contains(&pow_hash);
        let valid_tx = self.execution.verify_transaction(&transaction);

        println!("Valid mempool tx valid_tx:{} && {}", valid_tx, pow_not_yet_used_in_view);
        let is_valid_mempool_tx = valid_tx && pow_not_yet_used_in_view;
        return is_valid_mempool_tx;
    }

    fn genesis_block(&self) -> NotarizedBlock {
        let block_data = BlockData {
            transactions: vec![],
            height: 0,
            previous_block_hash: Sha256Digest::from(*b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        };

        let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);
        let block_hash = block_data.calculate_block_hash();
        let signature = private_key.sign_hash(block_hash);
        let notarization = Notarization { validator_index: 0, signature: signature.clone() };

        return NotarizedBlock {
            transactions: block_data.transactions,
            height: block_data.height,
            previous_block_hash: block_data.previous_block_hash,
            slot_leader_signature: signature,
            votes: vec![notarization],
        };
    }
    fn genesis_epoch_state(&self) -> EpochState {
        let mut epoch_state = EpochState::new();
        epoch_state.add_registered_validator(ed25519::PrivateKey::from_seed(1).public_key(), 100);
        epoch_state.add_registered_validator(ed25519::PrivateKey::from_seed(2).public_key(), 100);
        epoch_state.add_registered_validator(ed25519::PrivateKey::from_seed(3).public_key(), 100);
        epoch_state.add_registered_validator(ed25519::PrivateKey::from_seed(4).public_key(), 100);
        epoch_state.add_registered_validator(ed25519::PrivateKey::from_seed(5).public_key(), 100);

        return epoch_state;
    }
    fn get_epoch_state_for_slot(&self, _slot_height: u64) -> Option<EpochState> {
        return self.epoch_states.first().cloned();
    }

    fn get_local_validator_state_for_slot(&mut self, height: u64) -> &mut ValidatorSlotState {
        let validator_state =
            self.local_validator_slot_state.entry(height).or_insert(ValidatorSlotState {
                vote_state: VoteState::NoVoteYet,
                leader_state: LeaderState::HasNotProposedBlock,
            });

        return validator_state;
    }
    fn get_leader_for_slot(&self, height: u64) -> Option<ValidatorData> {
        //not secure
        //alts,
        //actual vrf
        //vulnerable vrf, just do blockhash
        //queue based
        //will probs do vrf in future, just do vulnerable "random" for now?
        let epoch_state = self.get_epoch_state_for_slot(height);

        //leader selection assume index
        if let Some(epoch_state) = epoch_state {
            let amount_of_validators = epoch_state.validator_data.len() as u64;
            let chosen_validator_index = height % amount_of_validators;

            let chosen_validator_pub_key =
                epoch_state.validator_index_to_key.get(&chosen_validator_index).unwrap();

            let chosen_validator = epoch_state.validator_data.get(chosen_validator_pub_key);
            return Some(chosen_validator.unwrap().clone());
        }
        return None;
    }
    fn is_leader_for_slot(&self, public_key: &ed25519::PublicKey, height: u64) -> bool {
        if let Some(leader) = self.get_leader_for_slot(height) {
            return leader.pub_key == *public_key;
        }
        return false;
    }
    fn get_slot_consensus_result(&mut self, height: u64) -> Consensus {
        let slot_state =
            self.slot_consensus.entry(height).or_insert(SlotConsensusState::new(height));
        return slot_state.consensus.clone();
    }
    fn get_block_to_propose(&mut self, slot_height: u64) -> Option<Block> {
        if let Some(previous_block_hash) = self.select_parent(slot_height) {
            let mut transactions = vec![];
            //todo check for duplicates within this block, also for block_verify
            for (tx_hash, transaction) in self.mempool.iter() {
                let not_in_viewchain_yet = self.view_consumed_txs.get(tx_hash).is_none();
                let pow_hash = &transaction.calculate_pow_hash();
                let pow_not_in_viewchain_yet =
                    self.view_consumed_pow_hashes.get(pow_hash).is_none();

                let valid_transaction = not_in_viewchain_yet && pow_not_in_viewchain_yet;
                if valid_transaction {
                    transactions.push(transaction.clone());
                }
            }

            let block_data = BlockData {
                height: slot_height,
                transactions: transactions,
                previous_block_hash: previous_block_hash,
            };
            let block_hash = block_data.calculate_block_hash();
            let signature = self.private_key.sign_hash(block_hash);
            let block = Block {
                height: block_data.height,
                transactions: block_data.transactions,
                previous_block_hash: block_data.previous_block_hash,
                slot_leader_signature: signature,
            };
            return Some(block);
        }
        return None;
    }
    fn get_slot_block_proposal(&self, height: u64) -> Option<&Block> {
        if let Some(slot_state) = self.slot_consensus.get(&height) {
            if let Some(block) = &slot_state.first_leader_signed_block_received {
                return Some(&block);
            } else {
                //otherwise check unverified blocks and try to verify them
                for block in slot_state.unverified_blocks_received.values() {
                    let block_verification = self.verify_block(&block);
                    if let BlockVerificationResult::Verifiable(block_verified) = block_verification
                    {
                        if block_verified {
                            return Some(&block);
                        }
                    }
                }
            }
        }
        return None;
    }
    fn has_block_proposal_for_slot(&self, height: u64) -> bool {
        return self.get_slot_block_proposal(height).is_some();
    }
    fn get_block(&mut self, height: u64, block_hash: &Sha256Digest) -> Option<Block> {
        if let Some(slot_state) = self.slot_consensus.get(&height) {
            if let Some(block) = &slot_state.first_leader_signed_block_received {
                let same_block_hash = block.calculate_block_hash() == *block_hash;
                if same_block_hash {
                    return Some(block.clone());
                }
            } else {
                //otherwise check unverified blocks and try to verify them
                for block in slot_state.unverified_blocks_received.values() {
                    let block_verification = self.verify_block(&block);

                    if let BlockVerificationResult::Verifiable(correct_block) = block_verification {
                        let same_block_hash = block.calculate_block_hash() == *block_hash;
                        if !same_block_hash {
                            panic!("block hash did not match");
                        }
                        if correct_block && same_block_hash {
                            return Some(block.clone());
                        }
                    }
                }
            }
        }
        return None;
    }
    fn has_stake_in_current_epoch(
        &self,
        validator_public_key: &ed25519::PublicKey,
        height: u64,
    ) -> bool {
        if let Some(epoch_state) = self.get_epoch_state_for_slot(height) {
            let has_stake_in_current_epoch =
                epoch_state.validator_data.contains_key(validator_public_key);
            return has_stake_in_current_epoch;
        } else {
            return false;
        }
    }
    fn get_validator_index(
        &self,
        public_key: &ed25519::PublicKey,
        slot_height: u64,
    ) -> Option<u64> {
        let epoch_state = self.get_epoch_state_for_slot(slot_height);
        if let Some(epoch_state) = epoch_state {
            if let Some(val) = epoch_state.validator_data.get(public_key) {
                return Some(val.validator_index);
            }
            return None;
        }
        return None;
    }
    fn get_validator_slot_vote_state(
        &mut self,
        slot_height: u64,
        validator_index: u64,
    ) -> &mut VoteState {
        let slot_consensus_state = self.get_slot_consensus_or_default(slot_height);
        let validator_slot_vote_state = slot_consensus_state
            .consumed_validator_indexes
            .entry(validator_index)
            .or_insert(VoteState::NoVoteYet);
        return validator_slot_vote_state;
    }
    fn get_slot_consensus_or_default(&mut self, slot_height: u64) -> &mut SlotConsensusState {
        let slot_consensus_state =
            self.slot_consensus.entry(slot_height).or_insert(SlotConsensusState::new(slot_height));
        return slot_consensus_state;
    }
    fn get_block_candidate_or_default(
        &mut self,
        slot_height: u64,
        block_hash: &Sha256Digest,
    ) -> &mut BlockCandidateState {
        let slot_consensus_state = self.get_slot_consensus_or_default(slot_height);
        let block_candidate = slot_consensus_state
            .block_candidates
            .entry(block_hash.clone())
            .or_insert(BlockCandidateState { voted_stake: 0, votes: BTreeMap::new() });
        return block_candidate;
    }
    fn select_parent(&self, slot_height: u64) -> Option<Sha256Digest> {
        let mut i = 1;
        loop {
            let slot = self.viewchain.get(&(slot_height - i));

            let Some(slot) = slot else {
                return None;
            };
            if let SlotState::Block(notarized_block) = slot {
                return Some(notarized_block.calculate_block_hash());
            }

            i += 1;
        }
    }
    fn verify_block(&self, block: &Block) -> BlockVerificationResult {
        let Some(leader_for_slot) = self.get_leader_for_slot(block.height) else {
            return BlockVerificationResult::CouldNotVerifyDefer;
        };
        let Some(previous_block_hash) = self.select_parent(block.height) else {
            info!(
                "verifyblock : could not verify parent blockhash for block {} because state not current, storing block and defering verification to when state is updated",
                block.height
            );
            //no parent in viewchain
            return BlockVerificationResult::CouldNotVerifyDefer;
        };
        let block_hash = block.calculate_block_hash();

        let signed_by_leader =
            leader_for_slot.pub_key.verify_signature_hash(block_hash, &block.slot_leader_signature);
        let previous_block_hash_match = previous_block_hash == block.previous_block_hash;

        let correct_block = signed_by_leader && previous_block_hash_match;
        return BlockVerificationResult::Verifiable(correct_block);
    }
    pub fn calculate_null_hash(height: u64) -> Sha256Digest {
        /*let name_space = r#"
      '
   \  :  /
`. __/ \__ .'
_ _\     /_ _
   /_   _\
 .'  \ /  `.    
   /  :  \    
      '
"#;*/
        let name_space = *b"nullification_domain";

        let null_digest = NullificationDigest { namespace: name_space, slot_height: height };
        let hash = null_digest.calculate_hash();
        return hash;
    }
    fn _update_consensus_state(&mut self, slot_height: u64) -> Consensus {
        let Some(epoch_state) = self.get_epoch_state_for_slot(slot_height) else {
            return Consensus::None;
        };
        let slot_consensus_state = self.get_slot_consensus_or_default(slot_height);
        let consensus_state_result = ValidatorStateMachine::check_consensus_for_slot(
            &slot_consensus_state,
            epoch_state.total_validator_stake,
        );
        slot_consensus_state.consensus = consensus_state_result.clone();
        return consensus_state_result;
    }

    async fn start_services(
        &mut self,
        addr: SocketAddr,
        peers: Vec<NodeRecord>,
        run_rpc_server: bool,
    ) {
        let rpc_server = RPCServer::new(self.networking.clone());
        if run_rpc_server {
            self.deploy_sites();
            tokio::spawn(async move {
                let _res = rpc_server.start_indexer_server().await;
            });
        }
        self.networking.run(addr.clone(), peers, self.database.clone()).await;
        //give time for network to connect
        sleep(Duration::from_secs(5)).await;
    }
    fn download_100_notarization_from_peer(&mut self, height: u64) {
        let networking = self.networking.clone();
        let block_vote_tx = self.block_vote_tx.clone();
        let null_vote_tx = self.nullification_vote_tx.clone();
        tokio::spawn(async move {
            let res = networking.get_100_notarization(height).await;

            //feed notarization into state
            if let Some(res) = res {
                for note in res.notarizations {
                    if note.notarization_type == GetNotarizationNotarizationType::Block {
                        for vote in note.votes {
                            let block_vote = BlockVote {
                                block_hash: note.hash,
                                slot_height: note.height,
                                signature: vote.signature,
                                validator_index: vote.validator_index,
                            };
                            let _res = block_vote_tx.send(block_vote);
                        }
                    } else if note.notarization_type
                        == GetNotarizationNotarizationType::Nullification
                    {
                        for vote in note.votes {
                            let null_vote = NullVote {
                                slot_height: note.height,
                                signature: vote.signature,
                                validator_index: vote.validator_index,
                            };
                            let _res = null_vote_tx.send(null_vote);
                        }
                    } else if note.notarization_type == GetNotarizationNotarizationType::NoneYet {
                    }
                }
            }
        });
    }
    fn download_100_blocks_from_peer(&mut self, height: u64) {
        let networking = self.networking.clone();
        let proposed_block_tx = self.proposedblock_tx.clone();
        tokio::spawn(async move {
            let blocks = networking.get_100_block_from_height(height).await;
            for block in blocks {
                let _res = proposed_block_tx.send(block);
            }
        });
    }

    pub fn _tests_init_state(
        &mut self,
        genesis_block: NotarizedBlock,
        genesis_epoch_state: EpochState,
    ) {
        self.viewchain.insert(0, SlotState::Block(genesis_block));
        self.epoch_states.push(genesis_epoch_state);
    }
    pub fn new(
        private_key: ed25519::PrivateKey,
        p2p_key: ed25519::PrivateKey,
        addr: SocketAddr,
    ) -> ValidatorStateMachine {
        let (block_vote_tx, block_vote_rx) = mpsc::unbounded_channel::<BlockVote>();
        let (null_vote_tx, null_vote_rx) = mpsc::unbounded_channel::<NullVote>();
        let (proposedblock_tx, proposedblock_rx) = mpsc::unbounded_channel::<Block>();

        let (transaction_tx, transaction_rx) = mpsc::unbounded_channel::<Transaction>();

        return ValidatorStateMachine {
            slot_consensus: HashMap::new(),
            slot_start: Instant::now(),
            last_time_asked_for_votes: Instant::now(),
            last_time_rebroadcast_votes: Instant::now(),
            viewchain: BTreeMap::new(),
            view_consumed_pow_hashes: HashSet::new(),
            view_consumed_txs: HashSet::new(),
            latest_finalized_block_height: 0,
            latest_non_cancellable_slot_height: 0,
            latest_view_height: 0,
            epoch_states: vec![],
            pub_key: private_key.public_key(),
            private_key: private_key,
            local_validator_slot_state: HashMap::new(),
            mempool: BTreeMap::new(),
            networking: Arc::new(Networking::new(
                block_vote_tx.clone(),
                null_vote_tx.clone(),
                proposedblock_tx.clone(),
                transaction_tx,
                p2p_key,
                addr.port(),
            )),
            database: Arc::new(BlockchainDatabase::new()),
            block_vote_rx: block_vote_rx,
            nullification_vote_rx: null_vote_rx,
            proposedblock_rx: proposedblock_rx,
            block_vote_tx: block_vote_tx.clone(),
            nullification_vote_tx: null_vote_tx.clone(),
            proposedblock_tx: proposedblock_tx.clone(),
            execution: Execution::new(),
            transactions_rx: transaction_rx,
            node_id: std::env::var("NODE_ID").unwrap().parse::<u16>().unwrap(),
        };
    }

    pub fn debug_write_blockchain(&self) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();

        let current_second = now.as_secs();
        if current_second % 5 == 0 {
            let filename = format!("./logs/blockchain{}.txt", self.node_id);
            let _res = std::fs::create_dir_all("logs");
            let mut output = "".to_string();

            for height in 0..=self.latest_view_height {
                let slot = self.database.read_slot(height);
                if let Some(slot) = slot {
                    output = format!("{}\n\n Block height {}", output, height);

                    if let SlotState::Block(block) = slot {
                        output = format!("{}\n Block {:#?}", output, block.calculate_block_hash());
                        output = format!(
                            "{}\n previous_block_hash {:#?}",
                            output, block.previous_block_hash
                        );
                        output = format!("{}\n votes {:#?}", output, block.votes);

                        output = format!("{}\n transactions:", output);
                        for transaction in block.transactions {
                            output = format!(
                                "{}\n transaction {:#?}",
                                output,
                                transaction.calculate_txhash()
                            );
                        }
                    } else if let SlotState::Nullification(null) = slot {
                        output = format!("{}\n Nullification {:#?}", output, null.votes);
                    }
                }
            }

            let file = OpenOptions::new()
                .append(false)
                .write(true)
                .create(true)
                .truncate(true)
                .open(filename)
                .unwrap();
            let mut writer = BufWriter::new(file);
            let _res = writeln!(writer, "{output}");
        }
    }
    pub fn deploy_sites(&mut self) {
        let deploy_forum_transactions = deploy_forum_transactions();
        //let deploy_vastrum_docs_transactions = deploy_docs_transactions();
        //let transactions = [deploy_forum_transactions, deploy_vastrum_docs_transactions].concat();
        let transactions = deploy_forum_transactions;
        for transaction in transactions {
            self.handle_new_mempool_tx(transaction);
        }
    }
    async fn sleep(&self) {
        let freeze_enabled = option_env!("FREEZE_NODES_RANDOMLY").is_some();
        if freeze_enabled && rand::thread_rng().r#gen::<f64>() < 0.01 {
            sleep(Duration::from_secs(10)).await;
        }
        sleep(Duration::from_millis(50)).await;
    }
    fn debug_view_info(&self, height: u64) {
        info!("self.latest_view_height {}", self.latest_view_height);
        if let Some(slot_consensus) = self.slot_consensus.get(&height) {
            info!("{slot_consensus:#?}");
        }
        info!("viewchain {:#?}", self.viewchain);
    }
}
pub struct ValidatorStateMachine {
    slot_consensus: HashMap<SlotHeight, SlotConsensusState>,

    slot_start: Instant,
    last_time_asked_for_votes: Instant,
    last_time_rebroadcast_votes: Instant,

    viewchain: BTreeMap<u64, SlotState>,
    view_consumed_pow_hashes: HashSet<Sha256Digest>, //to avoid allowing the same pow hash twice in viewchain
    view_consumed_txs: HashSet<Sha256Digest>,

    //view chain starts from here (root of view chain)
    latest_finalized_block_height: u64,
    //latest finalized block or latest nullification
    latest_non_cancellable_slot_height: u64,
    //view chain ends here (tip of chain)
    latest_view_height: u64,

    epoch_states: Vec<EpochState>,

    pub_key: ed25519::PublicKey,
    private_key: ed25519::PrivateKey,
    local_validator_slot_state: HashMap<u64, ValidatorSlotState>,
    mempool: BTreeMap<Sha256Digest, Transaction>, //TODO: in memory mempool, limit max to 100mb?
    transactions_rx: UnboundedReceiver<Transaction>,

    networking: Arc<Networking>,
    database: Arc<BlockchainDatabase>,
    execution: Execution,

    block_vote_rx: UnboundedReceiver<BlockVote>,
    nullification_vote_rx: UnboundedReceiver<NullVote>,
    proposedblock_rx: UnboundedReceiver<Block>,

    block_vote_tx: UnboundedSender<BlockVote>,
    nullification_vote_tx: UnboundedSender<NullVote>,
    proposedblock_tx: UnboundedSender<Block>,

    node_id: u16,
}
#[derive(Debug, Clone)]
pub struct EpochState {
    //TODO: these two need to be synced, better way to implement two key btreemap?
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
    pub fn get_validator_data(&self, validator_index: u64) -> Option<&ValidatorData> {
        let Some(validator_public_key) = self.validator_index_to_key.get(&validator_index) else {
            return None;
        };
        let validator_data = self.validator_data.get(validator_public_key).expect("invariant");
        return Some(validator_data);
    }
    pub fn add_registered_validator(&mut self, pub_key: ed25519::PublicKey, stake: u64) {
        let current_length = self.validator_data.len();
        let validator_index = current_length as u64;
        self.validator_data.insert(
            pub_key.clone(),
            ValidatorData {
                stake: stake,
                validator_index: validator_index,
                pub_key: pub_key.clone(),
            },
        );
        self.validator_index_to_key.insert(validator_index, pub_key);
        self.total_validator_stake += stake;
    }
}

#[derive(PartialEq, Debug)]
pub struct ValidatorSlotState {
    vote_state: VoteState,
    leader_state: LeaderState,
}

pub type SlotHeight = u64;
pub type ValidatorIndex = u64;
#[derive(PartialEq, Debug)]
pub enum VoteState {
    NoVoteYet,
    VotedForBlock(Sha256Digest),
    VotedForNullification,
}

#[derive(PartialEq, Debug)]
pub enum BlockVerificationResult {
    Verifiable(bool),
    CouldNotVerifyDefer,
}

#[derive(Debug)]
pub struct BlockCandidateState {
    voted_stake: u64,
    pub votes: BTreeMap<ValidatorIndex, ed25519::Signature>,
}
#[derive(Debug)]
pub struct SlotConsensusState {
    pub block_candidates: HashMap<Sha256Digest, BlockCandidateState>,
    pub null_stake: u64,
    pub nullification_votes: BTreeMap<ValidatorIndex, ed25519::Signature>,
    pub slot_height: u64,
    pub consumed_validator_indexes: HashMap<ValidatorIndex, VoteState>,
    pub consensus: Consensus,

    pub first_leader_signed_block_received: Option<Block>,
    pub unverified_blocks_received: HashMap<Sha256Digest, Block>,
}

impl SlotConsensusState {
    pub fn new(slot_height: u64) -> SlotConsensusState {
        return SlotConsensusState {
            block_candidates: HashMap::new(),
            null_stake: 0,
            nullification_votes: BTreeMap::new(),
            slot_height: slot_height,
            consumed_validator_indexes: HashMap::new(),
            consensus: Consensus::None,
            first_leader_signed_block_received: None,
            unverified_blocks_received: HashMap::new(),
        };
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct ValidatorData {
    pub stake: u64,
    pub validator_index: u64,
    pub pub_key: ed25519::PublicKey,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Consensus {
    FinalizedBlock(BlockNotarization),
    FinalizedNullification(NullificationNotarization),
    ImplicitNullification,
    ViewBlock(BlockNotarization),
    None,
}

#[derive(Clone, PartialEq, Debug)]
pub struct BlockNotarization {
    votes: BTreeMap<ValidatorIndex, ed25519::Signature>,
    block_hash: Sha256Digest,
    height: u64,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NullificationNotarization {
    votes: BTreeMap<ValidatorIndex, ed25519::Signature>,
    height: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HandleBlockVoteResult {
    ErrorInvalidSignature,
    ErrorEpochStateDoesNotYetExistForSlot,
    ErrorCouldNotFindValidatorIndex,
    ErrorValidatorHasAlreadyVotedInThisSlot,
    ErrorSlotAlreadyFinalized,
    Success(Consensus),
}
#[derive(Clone, Debug, PartialEq)]
pub enum HandleNullificationVoteResult {
    ErrorInvalidSignature,
    ErrorEpochStateDoesNotYetExistForSlot,
    ErrorCouldNotFindValidatorIndex,
    ErrorValidatorHasAlreadyVotedNullInThisSlot,
    ErrorSlotAlreadyFinalized,
    Success(Consensus),
}

#[derive(PartialEq, Debug)]
pub enum LeaderState {
    HasProposedBlock,
    HasNotProposedBlock,
}
#[cfg(test)]
mod tests_calculate_slot_consensus {
    use super::*;
    #[test]
    fn test_check_consensus_for_slot() {
        let mut slot_consensus_state = SlotConsensusState::new(0);

        let block_hash = Sha256Digest::from_u64(0);
        let total_validator_stake = 100;

        slot_consensus_state
            .block_candidates
            .insert(block_hash, BlockCandidateState { voted_stake: 1, votes: BTreeMap::new() });
        assert_eq!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::None
        );

        slot_consensus_state
            .block_candidates
            .insert(block_hash, BlockCandidateState { voted_stake: 30, votes: BTreeMap::new() });
        assert_eq!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::None
        );

        slot_consensus_state
            .block_candidates
            .insert(block_hash, BlockCandidateState { voted_stake: 77, votes: BTreeMap::new() });
        assert!(matches!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::ViewBlock(_)
        ));

        slot_consensus_state
            .block_candidates
            .insert(block_hash, BlockCandidateState { voted_stake: 81, votes: BTreeMap::new() });

        assert!(matches!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::FinalizedBlock(_)
        ));
    }
    #[test]
    fn test_check_nullification() {
        let mut slot_consensus_state = SlotConsensusState::new(0);

        let block_hash = Sha256Digest::from_u64(0);
        let total_validator_stake = 100;

        slot_consensus_state
            .block_candidates
            .insert(block_hash, BlockCandidateState { voted_stake: 1, votes: BTreeMap::new() });
        assert_eq!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::None
        );

        slot_consensus_state
            .block_candidates
            .insert(block_hash, BlockCandidateState { voted_stake: 30, votes: BTreeMap::new() });
        assert_eq!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::None
        );

        slot_consensus_state
            .block_candidates
            .insert(block_hash, BlockCandidateState { voted_stake: 55, votes: BTreeMap::new() });
        assert!(matches!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::ViewBlock(_)
        ));

        slot_consensus_state.null_stake = 39;
        assert!(matches!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::ViewBlock(_)
        ));

        slot_consensus_state.null_stake = 42;
        assert!(matches!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::FinalizedNullification(_)
        ));
    }
    #[test]
    fn test_check_implicit_nullification() {
        let mut slot_consensus_state = SlotConsensusState::new(0);

        let block_hash = Sha256Digest::from_u64(0);
        let total_validator_stake = 100;

        slot_consensus_state
            .block_candidates
            .insert(block_hash, BlockCandidateState { voted_stake: 1, votes: BTreeMap::new() });
        assert_eq!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::None
        );

        slot_consensus_state
            .block_candidates
            .insert(block_hash, BlockCandidateState { voted_stake: 30, votes: BTreeMap::new() });
        assert_eq!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::None
        );

        slot_consensus_state
            .block_candidates
            .insert(block_hash, BlockCandidateState { voted_stake: 55, votes: BTreeMap::new() });
        assert!(matches!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::ViewBlock(_)
        ));

        slot_consensus_state.null_stake = 39;
        assert!(matches!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::ViewBlock(_)
        ));

        slot_consensus_state.block_candidates.insert(
            Sha256Digest::from_u64(0),
            BlockCandidateState { voted_stake: 2, votes: BTreeMap::new() },
        );
        assert!(matches!(
            ValidatorStateMachine::check_consensus_for_slot(
                &slot_consensus_state,
                total_validator_stake
            ),
            Consensus::ImplicitNullification
        ));
    }
}
#[cfg(test)]
mod tests_handle_vote {
    use super::*;
    pub fn genesis_block() -> NotarizedBlock {
        let block_data = BlockData {
            transactions: vec![],
            height: 0,
            previous_block_hash: Sha256Digest::from(*b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        };
        let block_hash = block_data.calculate_block_hash();
        let signature = ed25519::PrivateKey::from_seed(100).sign_hash(block_hash);
        let notarization = Notarization { validator_index: 0, signature: signature.clone() };
        return NotarizedBlock {
            transactions: block_data.transactions,
            height: block_data.height,
            previous_block_hash: block_data.previous_block_hash,
            slot_leader_signature: signature,
            votes: vec![notarization],
        };
    }
    struct Validator {
        private_key: ed25519::PrivateKey,
        validator_index: u64,
        stake: u64,
    }
    #[test]
    fn test_check_block_vote_finalization() {
        unsafe {
            std::env::set_var("NODE_ID", "9999");
        }

        let validator_1 = Validator {
            private_key: ed25519::PrivateKey::from_seed(1),
            validator_index: 0,
            stake: 20,
        };
        let validator_2 = Validator {
            private_key: ed25519::PrivateKey::from_seed(2),
            validator_index: 1,
            stake: 20,
        };
        let validator_3 = Validator {
            private_key: ed25519::PrivateKey::from_seed(3),
            validator_index: 2,
            stake: 60,
        };
        let not_valid_validator = Validator {
            private_key: ed25519::PrivateKey::from_seed(4),
            validator_index: 3,
            stake: 0,
        };

        let mut epoch_state = EpochState::new();
        epoch_state
            .add_registered_validator(validator_1.private_key.public_key(), validator_1.stake);
        epoch_state
            .add_registered_validator(validator_2.private_key.public_key(), validator_2.stake);
        epoch_state
            .add_registered_validator(validator_3.private_key.public_key(), validator_3.stake);
        assert_eq!(100, epoch_state.total_validator_stake);

        let mut validator_state_machine = ValidatorStateMachine::new(
            ed25519::PrivateKey::from_seed(0),
            ed25519::PrivateKey::from_seed(0),
            std::net::SocketAddr::V4(std::net::SocketAddrV4::new(
                std::net::Ipv4Addr::LOCALHOST,
                7979,
            )),
        );
        validator_state_machine._tests_init_state(genesis_block(), epoch_state);

        let block_hash = Sha256Digest::from_u64(0);
        assert_eq!(
            validator_state_machine.handle_block_vote(BlockVote {
                slot_height: 0,
                block_hash: block_hash,
                signature: not_valid_validator.private_key.sign_hash(block_hash),
                validator_index: not_valid_validator.validator_index,
            }),
            HandleBlockVoteResult::ErrorCouldNotFindValidatorIndex
        );

        assert_eq!(
            validator_state_machine.handle_block_vote(BlockVote {
                slot_height: 0,
                block_hash: block_hash,
                signature: validator_2.private_key.sign_hash(block_hash),
                validator_index: validator_1.validator_index,
            }),
            HandleBlockVoteResult::ErrorInvalidSignature
        );

        //no consensus
        assert_eq!(
            validator_state_machine.handle_block_vote(BlockVote {
                slot_height: 0,
                block_hash: block_hash,
                signature: validator_1.private_key.sign_hash(block_hash),
                validator_index: validator_1.validator_index,
            }),
            HandleBlockVoteResult::Success(Consensus::None)
        );

        //view
        assert_eq!(
            validator_state_machine.handle_block_vote(BlockVote {
                slot_height: 0,
                block_hash: block_hash,
                signature: validator_2.private_key.sign_hash(block_hash),
                validator_index: validator_2.validator_index,
            }),
            HandleBlockVoteResult::Success(Consensus::ViewBlock(BlockNotarization {
                votes: BTreeMap::from([
                    (validator_1.validator_index, validator_1.private_key.sign_hash(block_hash)),
                    (validator_2.validator_index, validator_2.private_key.sign_hash(block_hash))
                ]),
                block_hash: block_hash,
                height: 0
            }))
        );

        //finalized
        assert_eq!(
            validator_state_machine.handle_block_vote(BlockVote {
                slot_height: 0,
                block_hash: block_hash,
                signature: validator_3.private_key.sign_hash(block_hash),
                validator_index: validator_3.validator_index,
            }),
            HandleBlockVoteResult::Success(Consensus::FinalizedBlock(BlockNotarization {
                votes: BTreeMap::from([
                    (validator_1.validator_index, validator_1.private_key.sign_hash(block_hash)),
                    (validator_2.validator_index, validator_2.private_key.sign_hash(block_hash)),
                    (validator_3.validator_index, validator_3.private_key.sign_hash(block_hash))
                ]),
                block_hash: block_hash,
                height: 0
            }))
        );
    }
    #[test]
    fn test_check_null_vote_nullification() {
        unsafe {
            std::env::set_var("NODE_ID", "9999");
        }

        let validator_1 = Validator {
            private_key: ed25519::PrivateKey::from_seed(1),
            validator_index: 0,
            stake: 20,
        };
        let validator_2 = Validator {
            private_key: ed25519::PrivateKey::from_seed(2),
            validator_index: 1,
            stake: 20,
        };
        let validator_3 = Validator {
            private_key: ed25519::PrivateKey::from_seed(3),
            validator_index: 2,
            stake: 60,
        };
        let not_valid_validator = Validator {
            private_key: ed25519::PrivateKey::from_seed(4),
            validator_index: 3,
            stake: 0,
        };

        let mut epoch_state = EpochState::new();
        epoch_state
            .add_registered_validator(validator_1.private_key.public_key(), validator_1.stake);
        epoch_state
            .add_registered_validator(validator_2.private_key.public_key(), validator_2.stake);
        epoch_state
            .add_registered_validator(validator_3.private_key.public_key(), validator_3.stake);
        assert_eq!(100, epoch_state.total_validator_stake);

        let mut validator_state_machine = ValidatorStateMachine::new(
            ed25519::PrivateKey::from_seed(0),
            ed25519::PrivateKey::from_seed(0),
            std::net::SocketAddr::V4(std::net::SocketAddrV4::new(
                std::net::Ipv4Addr::LOCALHOST,
                7979,
            )),
        );
        validator_state_machine._tests_init_state(genesis_block(), epoch_state);

        let null_hash = ValidatorStateMachine::calculate_null_hash(0);
        assert_eq!(
            validator_state_machine.handle_nullification_vote(NullVote {
                slot_height: 0,
                signature: not_valid_validator.private_key.sign_hash(null_hash),
                validator_index: not_valid_validator.validator_index,
            }),
            HandleNullificationVoteResult::ErrorCouldNotFindValidatorIndex
        );

        assert_eq!(
            validator_state_machine.handle_nullification_vote(NullVote {
                slot_height: 0,
                signature: validator_2.private_key.sign_hash(null_hash),
                validator_index: validator_1.validator_index,
            }),
            HandleNullificationVoteResult::ErrorInvalidSignature
        );

        //no consensus
        assert_eq!(
            validator_state_machine.handle_nullification_vote(NullVote {
                slot_height: 0,
                signature: validator_1.private_key.sign_hash(null_hash),
                validator_index: validator_1.validator_index,
            }),
            HandleNullificationVoteResult::Success(Consensus::None)
        );

        //finalized null
        assert_eq!(
            validator_state_machine.handle_nullification_vote(NullVote {
                slot_height: 0,
                signature: validator_2.private_key.sign_hash(null_hash),
                validator_index: validator_2.validator_index,
            }),
            HandleNullificationVoteResult::Success(Consensus::FinalizedNullification(
                NullificationNotarization {
                    votes: BTreeMap::from([
                        (validator_1.validator_index, validator_1.private_key.sign_hash(null_hash)),
                        (validator_2.validator_index, validator_2.private_key.sign_hash(null_hash))
                    ]),
                    height: 0
                }
            ))
        );

        //finalized null
        assert_eq!(
            validator_state_machine.handle_nullification_vote(NullVote {
                slot_height: 0,
                signature: validator_3.private_key.sign_hash(null_hash),
                validator_index: validator_3.validator_index,
            }),
            HandleNullificationVoteResult::Success(Consensus::FinalizedNullification(
                NullificationNotarization {
                    votes: BTreeMap::from([
                        (validator_1.validator_index, validator_1.private_key.sign_hash(null_hash)),
                        (validator_2.validator_index, validator_2.private_key.sign_hash(null_hash)),
                        (validator_3.validator_index, validator_3.private_key.sign_hash(null_hash))
                    ]),
                    height: 0
                }
            ))
        );
    }
    #[test]
    fn test_check_vote_implicit_nullification() {
        unsafe {
            std::env::set_var("NODE_ID", "9999");
        }

        let validator_1 = Validator {
            private_key: ed25519::PrivateKey::from_seed(1),
            validator_index: 0,
            stake: 20,
        };
        let validator_2 = Validator {
            private_key: ed25519::PrivateKey::from_seed(2),
            validator_index: 1,
            stake: 20,
        };
        let validator_3 = Validator {
            private_key: ed25519::PrivateKey::from_seed(3),
            validator_index: 2,
            stake: 60,
        };

        let mut epoch_state = EpochState::new();
        epoch_state
            .add_registered_validator(validator_1.private_key.public_key(), validator_1.stake);
        epoch_state
            .add_registered_validator(validator_2.private_key.public_key(), validator_2.stake);
        epoch_state
            .add_registered_validator(validator_3.private_key.public_key(), validator_3.stake);
        assert_eq!(100, epoch_state.total_validator_stake);

        let mut validator_state_machine = ValidatorStateMachine::new(
            ed25519::PrivateKey::from_seed(0),
            ed25519::PrivateKey::from_seed(0),
            std::net::SocketAddr::V4(std::net::SocketAddrV4::new(
                std::net::Ipv4Addr::LOCALHOST,
                7979,
            )),
        );
        validator_state_machine._tests_init_state(genesis_block(), epoch_state);

        let null_hash = ValidatorStateMachine::calculate_null_hash(0);
        let block_hash_1 = Sha256Digest::from_u64(0);
        let block_hash_2 = Sha256Digest::from_u64(1);

        //no consensus
        assert_eq!(
            validator_state_machine.handle_nullification_vote(NullVote {
                slot_height: 0,
                signature: validator_1.private_key.sign_hash(null_hash),
                validator_index: validator_1.validator_index,
            }),
            HandleNullificationVoteResult::Success(Consensus::None)
        );

        //cannot change vote from null to block
        assert_eq!(
            validator_state_machine.handle_block_vote(BlockVote {
                slot_height: 0,
                block_hash: block_hash_1,
                signature: validator_1.private_key.sign_hash(block_hash_1),
                validator_index: validator_1.validator_index,
            }),
            HandleBlockVoteResult::ErrorValidatorHasAlreadyVotedInThisSlot
        );

        //no consensus
        assert_eq!(
            validator_state_machine.handle_block_vote(BlockVote {
                slot_height: 0,
                block_hash: block_hash_1,
                signature: validator_2.private_key.sign_hash(block_hash_1),
                validator_index: validator_2.validator_index,
            }),
            HandleBlockVoteResult::Success(Consensus::None)
        );

        //implicit nullification
        assert_eq!(
            validator_state_machine.handle_block_vote(BlockVote {
                slot_height: 0,
                block_hash: block_hash_2,
                signature: validator_3.private_key.sign_hash(block_hash_2),
                validator_index: validator_3.validator_index,
            }),
            HandleBlockVoteResult::Success(Consensus::ImplicitNullification)
        );

        //switch block vote to null vote > finalized nullification
        assert_eq!(
            validator_state_machine.handle_nullification_vote(NullVote {
                slot_height: 0,
                signature: validator_2.private_key.sign_hash(null_hash),
                validator_index: validator_2.validator_index,
            }),
            HandleNullificationVoteResult::Success(Consensus::FinalizedNullification(
                NullificationNotarization {
                    votes: BTreeMap::from([
                        (validator_1.validator_index, validator_1.private_key.sign_hash(null_hash)),
                        (validator_2.validator_index, validator_2.private_key.sign_hash(null_hash))
                    ]),
                    height: 0
                }
            ))
        );

        //cannot vote null again
        assert_eq!(
            validator_state_machine.handle_nullification_vote(NullVote {
                slot_height: 0,
                signature: validator_2.private_key.sign_hash(null_hash),
                validator_index: validator_2.validator_index,
            }),
            HandleNullificationVoteResult::ErrorValidatorHasAlreadyVotedNullInThisSlot
        );
    }
}

use crate::{
    consensus::{
        comms::{BlockVote, NullVote},
        types::{
            Block, BlockData, Notarization, NotarizedBlock, NotarizedNullification,
            NullificationDigest, SlotState,
        },
    },
    db::blockchain::BlockchainDatabase,
    execution::execution::Execution,
    p2p::{
        domon::NodeRecord, networking::Networking,
        types::getnotarization::GetNotarizationNotarizationType,
    },
    rpc::api::RPCServer,
    utils::forum_deployer::deploy_forum_transactions,
};
use rand::Rng;
use shared_types::{
    crypto::{ed25519, sha256::Sha256Digest},
    types::execution::transaction::Transaction,
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::OpenOptions,
    io::{BufWriter, Write},
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{Duration, Instant, sleep},
};
use tracing::info;
