//! The ledger shell connects the ABCI++ interface with the Anoma ledger app.
//!
//! Any changes applied before [`Shell::finalize_block`] might have to be
//! reverted, so any changes applied in the methods [`Shell::prepare_proposal`],
//! [`Shell::process_proposal`] must be also reverted (unless we can simply
//! overwrite them in the next block).
//! More info in <https://github.com/anoma/anoma/issues/362>.

use std::cmp::max;
use std::convert::{TryFrom, TryInto};
use std::mem;
use std::path::Path;
use std::str::FromStr;

use anoma::ledger::gas::BlockGasMeter;
use anoma::ledger::parameters::Parameters;
use anoma::ledger::pos::anoma_proof_of_stake::types::{
    ActiveValidator, ValidatorSetUpdate,
};
use anoma::ledger::pos::anoma_proof_of_stake::PosBase;
use anoma::ledger::pos::PosParams;
use anoma::ledger::storage::write_log::WriteLog;
use anoma::ledger::{ibc, parameters, pos};
use anoma::proto::{self, Tx};
use anoma::types::address::Address;
use anoma::types::storage::{BlockHash, BlockHeight, Key};
use anoma::types::time::{DateTime, DateTimeUtc, TimeZone, Utc};
use anoma::types::token::Amount;
use anoma::types::{address, key, token};
use borsh::BorshSerialize;
use itertools::Itertools;
use tendermint::block::Header;
use tendermint_proto::abci::{
    self, ConsensusParams, Evidence, ValidatorUpdate,
};
use tendermint_proto::types::EvidenceParams;
use thiserror::Error;
use tower_abci::{request, response};

use super::rpc;
use crate::config::genesis;
use crate::node::ledger::events::{Event, EventType};
use crate::node::ledger::rpc::PrefixValue;
use crate::node::ledger::shims::abcipp_shim_types::shim;
use crate::node::ledger::{protocol, storage, tendermint_node};
use crate::{config, wallet};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error removing the DB data: {0}")]
    RemoveDB(std::io::Error),
    #[error("chain ID mismatch: {0}")]
    ChainId(String),
    #[error("Error decoding a transaction from bytes: {0}")]
    TxDecoding(proto::Error),
    #[error("Error trying to apply a transaction: {0}")]
    TxApply(protocol::Error),
    #[error("Gas limit exceeding while applying transactions in block")]
    GasOverflow,
    #[error("{0}")]
    Tendermint(tendermint_node::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn reset(config: config::Ledger) -> Result<()> {
    // simply nuke the DB files
    let db_path = &config.db;
    match std::fs::remove_dir_all(&db_path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        res => res.map_err(Error::RemoveDB)?,
    };
    // reset Tendermint state
    tendermint_node::reset(config).map_err(Error::Tendermint)?;
    Ok(())
}

#[derive(Clone, Debug)]
pub enum MempoolTxType {
    /// A transaction that has not been validated by this node before
    NewTransaction,
    /// A transaction that has been validated at some previous level that may
    /// need to be validated again
    RecheckTransaction,
}

#[derive(Debug)]
pub struct Shell {
    /// The persistent storage
    pub(super) storage: storage::PersistentStorage,
    /// Gas meter for the current block
    gas_meter: BlockGasMeter,
    /// Write log for the current block
    write_log: WriteLog,
    /// Byzantine validators given from ABCI++ `prepare_proposal` are stored in
    /// this field. They will be slashed when we finalize the block.
    byzantine_validators: Vec<Evidence>,
}

impl Shell {
    /// Create a new shell from a path to a database and a chain id. Looks
    /// up the database with this data and tries to load the last state.
    pub fn new(db_path: impl AsRef<Path>, chain_id: String) -> Self {
        let mut storage = storage::open(db_path, chain_id);
        storage
            .load_last_state()
            .map_err(|e| {
                tracing::error!("Cannot load the last state from the DB {}", e);
            })
            .expect("PersistentStorage cannot be initialized");

        Self {
            storage,
            gas_meter: BlockGasMeter::default(),
            write_log: WriteLog::default(),
            byzantine_validators: vec![],
        }
    }

    /// Create a new genesis for the chain with specified id. This includes
    /// 1. A set of initial users and tokens
    /// 2. Setting up the validity predicates for both users and tokens
    /// 3. A matchmaker
    pub fn init_chain(
        &mut self,
        init: request::InitChain,
    ) -> Result<response::InitChain> {
        let mut response = response::InitChain::default();
        let (current_chain_id, _) = self.storage.get_chain_id();
        if current_chain_id != init.chain_id {
            return Err(Error::ChainId(format!(
                "Current chain ID: {}, Tendermint chain ID: {}",
                current_chain_id, init.chain_id
            )));
        }
        let genesis = genesis::genesis();

        // Initialize because there is no block
        let token_vp =
            std::fs::read("wasm/vp_token.wasm").expect("cannot load token VP");
        let user_vp =
            std::fs::read("wasm/vp_user.wasm").expect("cannot load user VP");

        // TODO load initial accounts from genesis

        // temporary account addresses for testing, generated by the
        // address.rs module
        let albert = Address::decode("a1qq5qqqqqg4znssfsgcurjsfhgfpy2vjyxy6yg3z98pp5zvp5xgersvfjxvcnx3f4xycrzdfkak0xhx")
            .expect("The genesis address shouldn't fail decoding");
        let bertha = Address::decode("a1qq5qqqqqxv6yydz9xc6ry33589q5x33eggcnjs2xx9znydj9xuens3phxppnwvzpg4rrqdpswve4n9")
            .expect("The genesis address shouldn't fail decoding");
        let christel = Address::decode("a1qq5qqqqqxsuygd2x8pq5yw2ygdryxs6xgsmrsdzx8pryxv34gfrrssfjgccyg3zpxezrqd2y2s3g5s")
            .expect("The genesis address shouldn't fail decoding");
        let users = vec![albert, bertha, christel];

        let tokens = address::tokens();
        for token in tokens.keys() {
            // default tokens VPs for testing
            let key = Key::validity_predicate(token);
            self.storage
                .write(&key, token_vp.to_vec())
                .expect("Unable to write token VP");
        }

        for (user, token) in users.iter().cartesian_product(tokens.keys()) {
            // default user VPs for testing
            self.storage
                .write(&Key::validity_predicate(user), user_vp.to_vec())
                .expect("Unable to write user VP");

            // default user's tokens for testing
            self.storage
                .write(
                    &token::balance_key(token, user),
                    Amount::whole(1_000_000)
                        .try_to_vec()
                        .expect("encode token amount"),
                )
                .expect("Unable to set genesis balance");

            // default user's public keys for testing
            let pk_key = key::ed25519::pk_key(user);
            let pk = wallet::defaults::key_of(user.encode()).public;
            self.storage
                .write(&pk_key, pk.try_to_vec().expect("encode public key"))
                .expect("Unable to set genesis user public key");
        }

        // Temporary for testing, we have a fixed matchmaker account.  This
        // account has a public key for signing matchmaker txs and verifying
        // their signatures in its VP. The VP is the same as the user's VP,
        // which simply checks the signature. We could consider using the
        // same key as the intent gossip's p2p key.
        let matchmaker = address::matchmaker();
        let matchmaker_pk = key::ed25519::pk_key(&matchmaker);
        self.storage
            .write(
                &matchmaker_pk,
                wallet::defaults::matchmaker_keypair()
                    .public
                    .try_to_vec()
                    .expect("encode public key"),
            )
            .expect("Unable to set genesis user public key");
        self.storage
            .write(&Key::validity_predicate(&matchmaker), user_vp.to_vec())
            .expect("Unable to write matchmaker VP");

        let ts: tendermint_proto::google::protobuf::Timestamp =
            init.time.expect("Missing genesis time");
        let initial_height = init
            .initial_height
            .try_into()
            .expect("Unexpected block height");
        // TODO hacky conversion, depends on https://github.com/informalsystems/tendermint-rs/issues/870
        let genesis_time: DateTimeUtc =
            (Utc.timestamp(ts.seconds, ts.nanos as u32)).into();

        parameters::init_genesis_storage(
            &mut self.storage,
            &genesis.parameters,
        );
        // Depends on parameters being initialized
        self.storage
            .init_genesis_epoch(
                initial_height,
                genesis_time,
                &genesis.parameters,
            )
            .expect("Initializing genesis epoch must not fail");

        #[cfg(feature = "dev")]
        let validators = vec![genesis.validator];
        #[cfg(not(feature = "dev"))]
        let validators = genesis.validators;

        // Write validators' VPs and non-staked tokens amount
        for validator in &validators {
            let addr = &validator.pos_data.address;
            // Write the VP
            // TODO replace with https://github.com/anoma/anoma/issues/25)
            self.storage
                .write(&Key::validity_predicate(addr), user_vp.to_vec())
                .expect("Unable to write user VP");
            // Validator account key
            let pk_key = key::ed25519::pk_key(addr);
            self.storage
                .write(
                    &pk_key,
                    validator
                        .account_key
                        .try_to_vec()
                        .expect("encode public key"),
                )
                .expect("Unable to set genesis user public key");
            // Account balance (tokens no staked in PoS)
            self.storage
                .write(
                    &token::balance_key(&address::xan(), addr),
                    validator
                        .non_staked_balance
                        .try_to_vec()
                        .expect("encode token amount"),
                )
                .expect("Unable to set genesis balance");
        }

        // PoS system depends on epoch being initialized
        let (current_epoch, _gas) = self.storage.get_current_epoch();
        pos::init_genesis_storage(
            &mut self.storage,
            &genesis.pos_params,
            validators.iter().map(|validator| &validator.pos_data),
            current_epoch,
        );
        ibc::init_genesis_storage(&mut self.storage);

        let evidence_params =
            self.get_evidence_params(&genesis.parameters, &genesis.pos_params);
        response.consensus_params = Some(ConsensusParams {
            evidence: Some(evidence_params),
            ..response.consensus_params.unwrap_or_default()
        });
        Ok(response)
    }

    /// Load the Merkle root hash and the height of the last committed block, if
    /// any. This is returned when ABCI sends an `info` request.
    pub fn last_state(&self) -> response::Info {
        let mut response = response::Info::default();
        let result = self.storage.get_state();
        match result {
            Some((root, height)) => {
                tracing::info!(
                    "Last state root hash: {}, height: {}",
                    root,
                    height
                );
                response.last_block_app_hash = root.0;
                response.last_block_height =
                    height.try_into().expect("Invalid block height");
            }
            None => {
                tracing::info!(
                    "No state could be found, chain is not initialized"
                );
            }
        };

        response
    }

    /// Uses `path` in the query to forward the request to the
    /// right query method and returns the result (which may be
    /// the default if `path` is not a supported string.
    /// INVARIANT: This method must be stateless.
    pub fn query(&self, query: request::Query) -> response::Query {
        use rpc::Path;
        match Path::from_str(&query.path) {
            Ok(path) => match path {
                Path::DryRunTx => self.dry_run_tx(&query.data),
                Path::Epoch => {
                    let (epoch, _gas) = self.storage.get_last_epoch();
                    let value = anoma::ledger::storage::types::encode(&epoch);
                    response::Query {
                        value,
                        ..Default::default()
                    }
                }
                Path::Value(storage_key) => {
                    self.read_storage_value(&storage_key)
                }
                Path::Prefix(storage_key) => {
                    self.read_storage_prefix(&storage_key)
                }
            },
            Err(err) => response::Query {
                code: 1,
                info: format!("RPC error: {}", err),
                ..Default::default()
            },
        }
    }

    /// Begin a new block.
    ///
    /// INVARIANT: Any changes applied in this method must be reverted if the
    /// proposal is rejected (unless we can simply overwrite them in the
    /// next block).
    pub fn prepare_proposal(
        &mut self,
        hash: BlockHash,
        header: Header,
        byzantine_validators: Vec<Evidence>,
    ) {
        let height = BlockHeight(header.height.into());

        // We can safely reset meter, because if the block is rejected, we'll
        // reset again on the next proposal, until the proposal is accepted
        self.gas_meter.reset();

        // The values set will be overwritten if this proposal is rejected.
        self.storage
            .begin_block(hash, height)
            .expect("Beginning a block shouldn't fail");

        // The value set will be overwritten if this proposal is rejected.
        self.storage
            .set_header(header)
            .expect("Setting a header shouldn't fail");

        // The value set will be overwritten if this proposal is rejected.
        self.byzantine_validators = byzantine_validators;
    }

    /// Apply PoS slashes from the evidence
    fn slash(&mut self) {
        if !self.byzantine_validators.is_empty() {
            let byzantine_validators =
                mem::take(&mut self.byzantine_validators);
            let pos_params = self.storage.read_pos_params();
            let current_epoch = self.storage.block.epoch;
            for evidence in byzantine_validators {
                let evidence_height = match u64::try_from(evidence.height) {
                    Ok(height) => height,
                    Err(err) => {
                        tracing::error!(
                            "Unexpected evidence block height {}",
                            err
                        );
                        continue;
                    }
                };
                let evidence_epoch = match self
                    .storage
                    .block
                    .pred_epochs
                    .get_epoch(BlockHeight(evidence_height))
                {
                    Some(epoch) => epoch,
                    None => {
                        tracing::error!(
                            "Couldn't find epoch for evidence block height {}",
                            evidence_height
                        );
                        continue;
                    }
                };
                let slash_type =
                    match abci::EvidenceType::from_i32(evidence.r#type) {
                        Some(r#type) => match r#type {
                            abci::EvidenceType::DuplicateVote => {
                                pos::types::SlashType::DuplicateVote
                            }
                            abci::EvidenceType::LightClientAttack => {
                                pos::types::SlashType::LightClientAttack
                            }
                            abci::EvidenceType::Unknown => {
                                tracing::error!(
                                    "Unknown evidence: {:#?}",
                                    evidence
                                );
                                continue;
                            }
                        },
                        None => {
                            tracing::error!(
                                "Unexpected evidence type {}",
                                evidence.r#type
                            );
                            continue;
                        }
                    };
                let validator_raw_hash = match evidence.validator {
                    Some(validator) => {
                        match String::from_utf8(validator.address) {
                            Ok(raw_hash) => raw_hash,
                            Err(err) => {
                                tracing::error!(
                                    "Evidence failed to decode validator \
                                     address from utf-8 with {}",
                                    err
                                );
                                continue;
                            }
                        }
                    }
                    None => {
                        tracing::error!(
                            "Evidence without a validator {:#?}",
                            evidence
                        );
                        continue;
                    }
                };
                let validator = match self
                    .storage
                    .read_validator_address_raw_hash(&validator_raw_hash)
                {
                    Some(validator) => validator,
                    None => {
                        tracing::error!(
                            "Cannot find validator's address from raw hash {}",
                            validator_raw_hash
                        );
                        continue;
                    }
                };
                tracing::info!(
                    "Slashing {} for {} in epoch {}, block height {}",
                    evidence_epoch,
                    slash_type,
                    validator,
                    evidence_height
                );
                if let Err(err) = self.storage.slash(
                    &pos_params,
                    current_epoch,
                    evidence_epoch,
                    evidence_height,
                    slash_type,
                    &validator,
                ) {
                    tracing::error!("Error in slashing: {}", err);
                }
            }
        }
    }

    /// INVARIANT: This method must be stateless.
    pub fn verify_header(
        &self,
        _req: shim::request::VerifyHeader,
    ) -> shim::response::VerifyHeader {
        Default::default()
    }

    /// Check the fees and signatures of the fee payer for a transaction
    ///
    /// INVARIANT: Any changes applied in this method must be reverted if the
    /// proposal is rejected (unless we can simply overwrite them in the
    /// next block).
    pub fn process_proposal(
        &mut self,
        _req: shim::request::ProcessProposal,
    ) -> shim::response::ProcessProposal {
        Default::default()
    }

    pub fn revert_proposal(
        &mut self,
        _req: shim::request::RevertProposal,
    ) -> shim::response::RevertProposal {
        Default::default()
    }

    /// INVARIANT: This method must be stateless.
    pub fn extend_vote(
        &self,
        _req: shim::request::ExtendVote,
    ) -> shim::response::ExtendVote {
        Default::default()
    }

    /// Validate and apply transactions.
    pub fn finalize_block(
        &mut self,
        req: shim::request::FinalizeBlock,
    ) -> Result<shim::response::FinalizeBlock> {
        let header = self
            .storage
            .header
            .as_ref()
            .expect("Header must have been set in prepare_proposal.");
        let height = BlockHeight(header.height.into());
        let time: DateTime<Utc> = header.time.into();
        let time: DateTimeUtc = time.into();
        let new_epoch = self
            .storage
            .update_epoch(height, time)
            .expect("Must be able to update epoch");

        self.slash();

        let mut response = shim::response::FinalizeBlock::default();
        for tx in &req.txs {
            let mut tx_result =
                Event::new_tx_event(EventType::Applied, tx, req.height);
            match protocol::apply_tx(
                tx,
                &mut self.gas_meter,
                &mut self.write_log,
                &self.storage,
            )
            .map_err(Error::TxApply)
            {
                Ok(result) => {
                    if result.is_accepted() {
                        tracing::info!(
                            "all VPs accepted apply_tx storage modification \
                             {:#?}",
                            result
                        );
                        self.write_log.commit_tx();
                        tx_result["code"] = "0".into();
                        match serde_json::to_string(
                            &result.initialized_accounts,
                        ) {
                            Ok(initialized_accounts) => {
                                tx_result["initialized_accounts"] =
                                    initialized_accounts;
                            }
                            Err(err) => {
                                tracing::error!(
                                    "Failed to serialize the initialized \
                                     accounts: {}",
                                    err
                                );
                            }
                        }
                    } else {
                        tracing::info!(
                            "some VPs rejected apply_tx storage modification \
                             {:#?}",
                            result.vps_result.rejected_vps
                        );
                        self.write_log.drop_tx();
                        tx_result["code"] = "1".into();
                    }
                    tx_result["gas_used"] = result.gas_used.to_string();
                    tx_result["info"] = result.to_string();
                }
                Err(msg) => {
                    tx_result["gas_used"] = self
                        .gas_meter
                        .get_current_transaction_gas()
                        .to_string();
                    tx_result["info"] = msg.to_string();
                    tx_result["code"] = "2".into();
                }
            }
            response.events.push(tx_result.into());
        }

        if new_epoch {
            // Apply validator set update
            let (current_epoch, _gas) = self.storage.get_current_epoch();
            // TODO ABCI validator updates on block H affects the validator set
            // on block H+2, do we need to update a block earlier?
            self.storage.validator_set_update(current_epoch, |update| {
                let (consensus_key, power) = match update {
                    ValidatorSetUpdate::Active(ActiveValidator {
                        consensus_key,
                        voting_power,
                    }) => {
                        let power: u64 = voting_power.into();
                        let power: i64 = power
                            .try_into()
                            .expect("unexpected validator's voting power");
                        (consensus_key, power)
                    }
                    ValidatorSetUpdate::Deactivated(consensus_key) => {
                        // Any validators that have become inactive must
                        // have voting power set to 0 to remove them from
                        // the active set
                        let power = 0_i64;
                        (consensus_key, power)
                    }
                };
                let consensus_key: ed25519_dalek::PublicKey =
                    consensus_key.into();
                let pub_key = tendermint_proto::crypto::PublicKey {
                    sum: Some(
                        tendermint_proto::crypto::public_key::Sum::Ed25519(
                            consensus_key.to_bytes().to_vec(),
                        ),
                    ),
                };
                let pub_key = Some(pub_key);
                let update = ValidatorUpdate { pub_key, power };
                response.validator_updates.push(update);
            });

            // Update evidence parameters
            let (parameters, _gas) = parameters::read(&self.storage)
                .expect("Couldn't read protocol parameters");
            let pos_params = self.storage.read_pos_params();
            let evidence_params =
                self.get_evidence_params(&parameters, &pos_params);
            response.consensus_param_updates = Some(ConsensusParams {
                evidence: Some(evidence_params),
                ..response.consensus_param_updates.unwrap_or_default()
            });
        }

        response.gas_used = self
            .gas_meter
            .finalize_transaction()
            .map_err(|_| Error::GasOverflow)?;
        Ok(response)
    }

    /// Commit a block. Persist the application state and return the Merkle root
    /// hash.
    pub fn commit(&mut self) -> response::Commit {
        let mut response = response::Commit::default();
        // commit changes from the write-log to storage
        self.write_log
            .commit_block(&mut self.storage)
            .expect("Expected committing block write log success");
        // store the block's data in DB
        self.storage.commit().unwrap_or_else(|e| {
            tracing::error!(
                "Encountered a storage error while committing a block {:?}",
                e
            )
        });

        let root = self.storage.merkle_root();
        tracing::info!(
            "Committed block hash: {}, height: {}",
            root,
            self.storage.last_height,
        );
        response.data = root.0;
        response
    }

    /// Validate a transaction request. On success, the transaction will
    /// included in the mempool and propagated to peers, otherwise it will be
    /// rejected.
    pub fn mempool_validate(
        &self,
        tx_bytes: &[u8],
        r#_type: MempoolTxType,
    ) -> response::CheckTx {
        let mut response = response::CheckTx::default();
        match Tx::try_from(tx_bytes).map_err(Error::TxDecoding) {
            Ok(_) => response.log = String::from("Mempool validation passed"),
            Err(msg) => {
                response.code = 1;
                response.log = msg.to_string();
            }
        }
        response
    }

    /// Simulate validation and application of a transaction.
    fn dry_run_tx(&self, tx_bytes: &[u8]) -> response::Query {
        let mut response = response::Query::default();
        let mut gas_meter = BlockGasMeter::default();
        let mut write_log = WriteLog::default();
        match protocol::apply_tx(
            tx_bytes,
            &mut gas_meter,
            &mut write_log,
            &self.storage,
        )
        .map_err(Error::TxApply)
        {
            Ok(result) => response.info = result.to_string(),
            Err(error) => {
                response.code = 1;
                response.log = format!("{}", error);
            }
        }
        response
    }

    /// Query to read a value from storage
    fn read_storage_value(&self, key: &Key) -> response::Query {
        match self.storage.read(key) {
            Ok((Some(value), _gas)) => response::Query {
                value,
                ..Default::default()
            },
            Ok((None, _gas)) => response::Query {
                code: 1,
                info: format!("No value found for key: {}", key),
                ..Default::default()
            },
            Err(err) => response::Query {
                code: 2,
                info: format!("Storage error: {}", err),
                ..Default::default()
            },
        }
    }

    /// Query to read a range of values from storage with a matching prefix. The
    /// value in successful response is a [`Vec<PrefixValue>`] encoded with
    /// [`BorshSerialize`].
    fn read_storage_prefix(&self, key: &Key) -> response::Query {
        let (iter, _gas) = self.storage.iter_prefix(key);
        let mut iter = iter.peekable();
        if iter.peek().is_none() {
            response::Query {
                code: 1,
                info: format!("No value found for key: {}", key),
                ..Default::default()
            }
        } else {
            let values: std::result::Result<
                Vec<PrefixValue>,
                anoma::types::storage::Error,
            > = iter
                .map(|(key, value, _gas)| {
                    let key = Key::parse(key)?;
                    Ok(PrefixValue { key, value })
                })
                .collect();
            match values {
                Ok(values) => {
                    let value = values.try_to_vec().unwrap();
                    response::Query {
                        value,
                        ..Default::default()
                    }
                }
                Err(err) => response::Query {
                    code: 1,
                    info: format!(
                        "Error parsing a storage key {}: {}",
                        key, err
                    ),
                    ..Default::default()
                },
            }
        }
    }

    fn get_evidence_params(
        &self,
        protocol_params: &Parameters,
        pos_params: &PosParams,
    ) -> EvidenceParams {
        // Minimum number of epochs before tokens are unbonded and can be
        // withdrawn
        let len_before_unbonded = max(pos_params.unbonding_len as i64 - 1, 0);
        let max_age_num_blocks: i64 =
            protocol_params.epoch_duration.min_num_of_blocks as i64
                * len_before_unbonded;
        let min_duration_secs =
            protocol_params.epoch_duration.min_duration.0 as i64;
        let max_age_duration =
            Some(tendermint_proto::google::protobuf::Duration {
                seconds: min_duration_secs * len_before_unbonded,
                nanos: 0,
            });
        EvidenceParams {
            max_age_num_blocks,
            max_age_duration,
            ..EvidenceParams::default()
        }
    }
}
