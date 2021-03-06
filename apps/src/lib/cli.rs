//! The CLI commands that are re-used between the executables `anoma`,
//! `anoma-node` and `anoma-client`.
//!
//! The `anoma` executable groups together the most commonly used commands
//! inlined from the node and the client. The other commands for the node or the
//! client can be dispatched via `anoma node ...` or `anoma client ...`,
//! respectively.

mod context;
mod utils;

use clap::{AppSettings, ArgMatches};
pub use utils::safe_exit;
use utils::*;

pub use self::context::Context;
use super::config;

const AUTHOR: &str = "Heliax AG <hello@heliax.dev>";
const APP_NAME: &str = "Anoma";
const CLI_VERSION: &str = "0.1.0";
const NODE_VERSION: &str = "0.1.0";
const CLIENT_VERSION: &str = "0.1.0";
const WALLET_VERSION: &str = "0.1.0";

// Main Anoma sub-commands
const NODE_CMD: &str = "node";
const CLIENT_CMD: &str = "client";
const WALLET_CMD: &str = "wallet";

pub mod cmds {
    use clap::AppSettings;

    use super::utils::*;
    use super::{args, ArgMatches, CLIENT_CMD, NODE_CMD, WALLET_CMD};

    /// Commands for `anoma` binary.
    #[allow(clippy::large_enum_variant)]
    #[derive(Clone, Debug)]
    pub enum Anoma {
        Node(AnomaNode),
        Client(AnomaClient),
        Wallet(AnomaWallet),
        // Inlined commands from the node and the client.
        Ledger(Ledger),
        Gossip(Gossip),
        TxCustom(TxCustom),
        TxTransfer(TxTransfer),
        TxUpdateVp(TxUpdateVp),
        Intent(Intent),
    }

    impl Cmd for Anoma {
        fn add_sub(app: App) -> App {
            app.subcommand(AnomaNode::def())
                .subcommand(AnomaClient::def())
                .subcommand(AnomaWallet::def())
                .subcommand(Ledger::def())
                .subcommand(Gossip::def())
                .subcommand(TxCustom::def())
                .subcommand(TxTransfer::def())
                .subcommand(TxUpdateVp::def())
                .subcommand(Intent::def())
        }

        fn parse(matches: &ArgMatches) -> Option<Self> {
            let node = SubCmd::parse(matches).map(Self::Node);
            let client = SubCmd::parse(matches).map(Self::Client);
            let wallet = SubCmd::parse(matches).map(Self::Wallet);
            let ledger = SubCmd::parse(matches).map(Self::Ledger);
            let gossip = SubCmd::parse(matches).map(Self::Gossip);
            let tx_custom = SubCmd::parse(matches).map(Self::TxCustom);
            let tx_transfer = SubCmd::parse(matches).map(Self::TxTransfer);
            let tx_update_vp = SubCmd::parse(matches).map(Self::TxUpdateVp);
            let intent = SubCmd::parse(matches).map(Self::Intent);
            node.or(client)
                .or(wallet)
                .or(ledger)
                .or(gossip)
                .or(tx_custom)
                .or(tx_transfer)
                .or(tx_update_vp)
                .or(intent)
        }
    }

    /// Used as top-level commands (`Cmd` instance) in `anoman` binary.
    /// Used as sub-commands (`SubCmd` instance) in `anoma` binary.
    #[derive(Clone, Debug)]
    #[allow(clippy::large_enum_variant)]
    pub enum AnomaNode {
        Ledger(Ledger),
        Gossip(Gossip),
        Config(Config),
    }

    impl Cmd for AnomaNode {
        fn add_sub(app: App) -> App {
            app.subcommand(Ledger::def())
                .subcommand(Gossip::def())
                .subcommand(Config::def())
        }

        fn parse(matches: &ArgMatches) -> Option<Self> {
            let ledger = SubCmd::parse(matches).map(Self::Ledger);
            let gossip = SubCmd::parse(matches).map(Self::Gossip);
            let config = SubCmd::parse(matches).map(Self::Config);
            ledger.or(gossip).or(config)
        }
    }
    impl SubCmd for AnomaNode {
        const CMD: &'static str = NODE_CMD;

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .and_then(|matches| <Self as Cmd>::parse(matches))
        }

        fn def() -> App {
            <Self as Cmd>::add_sub(
                App::new(Self::CMD)
                    .about("Node sub-commands")
                    .setting(AppSettings::SubcommandRequiredElseHelp),
            )
        }
    }

    /// Used as top-level commands (`Cmd` instance) in `anomac` binary.
    /// Used as sub-commands (`SubCmd` instance) in `anoma` binary.
    #[derive(Clone, Debug)]
    pub enum AnomaClient {
        // Ledger cmds
        TxCustom(TxCustom),
        TxTransfer(TxTransfer),
        TxUpdateVp(TxUpdateVp),
        TxInitAccount(TxInitAccount),
        Bond(Bond),
        Unbond(Unbond),
        Withdraw(Withdraw),
        QueryEpoch(QueryEpoch),
        QueryBalance(QueryBalance),
        QueryBonds(QueryBonds),
        QueryVotingPower(QueryVotingPower),
        QuerySlashes(QuerySlashes),
        // Gossip cmds
        Intent(Intent),
        SubscribeTopic(SubscribeTopic),
    }

    impl Cmd for AnomaClient {
        fn add_sub(app: App) -> App {
            app
                // Simple transactions
                .subcommand(TxCustom::def().display_order(1))
                .subcommand(TxTransfer::def().display_order(1))
                .subcommand(TxUpdateVp::def().display_order(1))
                .subcommand(TxInitAccount::def().display_order(1))
                // PoS transactions
                .subcommand(Bond::def().display_order(2))
                .subcommand(Unbond::def().display_order(2))
                .subcommand(Withdraw::def().display_order(2))
                // Queries
                .subcommand(QueryEpoch::def().display_order(3))
                .subcommand(QueryBalance::def().display_order(3))
                .subcommand(QueryBonds::def().display_order(3))
                .subcommand(QueryVotingPower::def().display_order(3))
                .subcommand(QuerySlashes::def().display_order(3))
                // Intents
                .subcommand(Intent::def().display_order(4))
                .subcommand(SubscribeTopic::def().display_order(4))
        }

        fn parse(matches: &ArgMatches) -> Option<Self> {
            let tx_custom = SubCmd::parse(matches).map(Self::TxCustom);
            let tx_transfer = SubCmd::parse(matches).map(Self::TxTransfer);
            let tx_update_vp = SubCmd::parse(matches).map(Self::TxUpdateVp);
            let tx_init_account =
                SubCmd::parse(matches).map(Self::TxInitAccount);
            let bond = SubCmd::parse(matches).map(Self::Bond);
            let unbond = SubCmd::parse(matches).map(Self::Unbond);
            let withdraw = SubCmd::parse(matches).map(Self::Withdraw);
            let query_epoch = SubCmd::parse(matches).map(Self::TxInitAccount);
            let query_balance = SubCmd::parse(matches).map(Self::QueryBalance);
            let query_bonds = SubCmd::parse(matches).map(Self::QueryBonds);
            let query_voting_power =
                SubCmd::parse(matches).map(Self::QueryVotingPower);
            let query_slashes = SubCmd::parse(matches).map(Self::QuerySlashes);
            let intent = SubCmd::parse(matches).map(Self::Intent);
            let subscribe_topic =
                SubCmd::parse(matches).map(Self::SubscribeTopic);
            tx_custom
                .or(tx_transfer)
                .or(tx_update_vp)
                .or(tx_init_account)
                .or(bond)
                .or(unbond)
                .or(withdraw)
                .or(query_epoch)
                .or(query_balance)
                .or(query_bonds)
                .or(query_voting_power)
                .or(query_slashes)
                .or(intent)
                .or(subscribe_topic)
        }
    }

    impl SubCmd for AnomaClient {
        const CMD: &'static str = CLIENT_CMD;

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .and_then(|matches| <Self as Cmd>::parse(matches))
        }

        fn def() -> App {
            <Self as Cmd>::add_sub(
                App::new(Self::CMD)
                    .about("Client sub-commands")
                    .setting(AppSettings::SubcommandRequiredElseHelp),
            )
        }
    }

    #[derive(Clone, Debug)]
    pub enum AnomaWallet {
        /// Key management commands
        Key(WalletKey),
        /// Address management commands
        Address(WalletAddress),
    }

    impl Cmd for AnomaWallet {
        fn add_sub(app: App) -> App {
            app.subcommand(WalletKey::def())
                .subcommand(WalletAddress::def())
        }

        fn parse(matches: &ArgMatches) -> Option<Self> {
            let key = SubCmd::parse(matches).map(Self::Key);
            let address = SubCmd::parse(matches).map(Self::Address);
            key.or(address)
        }
    }

    impl SubCmd for AnomaWallet {
        const CMD: &'static str = WALLET_CMD;

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .and_then(|matches| <Self as Cmd>::parse(matches))
        }

        fn def() -> App {
            <Self as Cmd>::add_sub(
                App::new(Self::CMD)
                    .about("Wallet sub-commands")
                    .setting(AppSettings::SubcommandRequiredElseHelp),
            )
        }
    }

    #[derive(Clone, Debug)]
    #[allow(clippy::large_enum_variant)]
    pub enum WalletKey {
        Gen(KeyGen),
        Find(KeyFind),
        List(KeyList),
        Export(Export),
    }

    impl SubCmd for WalletKey {
        const CMD: &'static str = "key";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).and_then(|matches| {
                let generate = SubCmd::parse(matches).map(Self::Gen);
                let lookup = SubCmd::parse(matches).map(Self::Find);
                let list = SubCmd::parse(matches).map(Self::List);
                let export = SubCmd::parse(matches).map(Self::Export);
                generate.or(lookup).or(list).or(export)
            })
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about(
                    "Keypair management, including methods to generate and \
                     look-up keys",
                )
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(KeyGen::def())
                .subcommand(KeyFind::def())
                .subcommand(KeyList::def())
                .subcommand(Export::def())
        }
    }

    /// Generate a new keypair and an implicit address derived from it
    #[derive(Clone, Debug)]
    pub struct KeyGen(pub args::KeyAndAddressGen);

    impl SubCmd for KeyGen {
        const CMD: &'static str = "gen";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| Self(args::KeyAndAddressGen::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about(
                    "Generates a keypair with a given alias and derive the \
                     implicit address from its public key. The address will \
                     be stored with the same alias.",
                )
                .add_args::<args::KeyAndAddressGen>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct KeyFind(pub args::KeyFind);

    impl SubCmd for KeyFind {
        const CMD: &'static str = "find";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| (Self(args::KeyFind::parse(matches))))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Searches for a keypair from a public key or an alias")
                .add_args::<args::KeyFind>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct KeyList(pub args::KeyList);

    impl SubCmd for KeyList {
        const CMD: &'static str = "list";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| (Self(args::KeyList::parse(matches))))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("List all known keys")
                .add_args::<args::KeyList>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct Export(pub args::KeyExport);

    impl SubCmd for Export {
        const CMD: &'static str = "export";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| (Self(args::KeyExport::parse(matches))))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Exports a keypair to a file")
                .add_args::<args::KeyExport>()
        }
    }

    #[derive(Clone, Debug)]
    pub enum WalletAddress {
        Gen(AddressGen),
        Find(AddressFind),
        List(AddressList),
        Add(AddressAdd),
    }

    impl SubCmd for WalletAddress {
        const CMD: &'static str = "address";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).and_then(|matches| {
                let gen = SubCmd::parse(matches).map(Self::Gen);
                let find = SubCmd::parse(matches).map(Self::Find);
                let list = SubCmd::parse(matches).map(Self::List);
                let add = SubCmd::parse(matches).map(Self::Add);
                gen.or(find).or(list).or(add)
            })
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about(
                    "Address management, including methods to generate and \
                     look-up addresses",
                )
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(AddressGen::def())
                .subcommand(AddressFind::def())
                .subcommand(AddressList::def())
                .subcommand(AddressAdd::def())
        }
    }

    /// Generate a new keypair and an implicit address derived from it
    #[derive(Clone, Debug)]
    pub struct AddressGen(pub args::KeyAndAddressGen);

    impl SubCmd for AddressGen {
        const CMD: &'static str = "gen";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).map(|matches| {
                AddressGen(args::KeyAndAddressGen::parse(matches))
            })
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about(
                    "Generates a keypair with a given alias and derive the \
                     implicit address from its public key. The address will \
                     be stored with the same alias.",
                )
                .add_args::<args::KeyAndAddressGen>()
        }
    }

    /// Find an address by its alias
    #[derive(Clone, Debug)]
    pub struct AddressFind(pub args::AddressFind);

    impl SubCmd for AddressFind {
        const CMD: &'static str = "find";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| AddressFind(args::AddressFind::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Find an address by its alias")
                .add_args::<args::AddressFind>()
        }
    }

    /// List known addresses
    #[derive(Clone, Debug)]
    pub struct AddressList;

    impl SubCmd for AddressList {
        const CMD: &'static str = "list";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|_matches| AddressList)
        }

        fn def() -> App {
            App::new(Self::CMD).about("List all known addresses")
        }
    }

    /// Generate a new keypair and an implicit address derived from it
    #[derive(Clone, Debug)]
    pub struct AddressAdd(pub args::AddressAdd);

    impl SubCmd for AddressAdd {
        const CMD: &'static str = "add";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| AddressAdd(args::AddressAdd::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Store an alias for an address in the wallet")
                .add_args::<args::AddressAdd>()
        }
    }

    #[derive(Clone, Debug)]
    pub enum Ledger {
        Run(LedgerRun),
        Reset(LedgerReset),
    }

    impl SubCmd for Ledger {
        const CMD: &'static str = "ledger";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).and_then(|matches| {
                let run = SubCmd::parse(matches).map(Self::Run);
                let reset = SubCmd::parse(matches).map(Self::Reset);
                run.or(reset)
                    // The `run` command is the default if no sub-command given
                    .or(Some(Self::Run(LedgerRun)))
            })
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about(
                    "Ledger node sub-commands. If no sub-command specified, \
                     defaults to run the node.",
                )
                .subcommand(LedgerRun::def())
                .subcommand(LedgerReset::def())
        }
    }

    #[derive(Clone, Debug)]
    pub struct LedgerRun;

    impl SubCmd for LedgerRun {
        const CMD: &'static str = "run";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).map(|_matches| Self)
        }

        fn def() -> App {
            App::new(Self::CMD).about("Run Anoma ledger node.")
        }
    }

    #[derive(Clone, Debug)]
    pub struct LedgerReset;

    impl SubCmd for LedgerReset {
        const CMD: &'static str = "reset";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).map(|_matches| Self)
        }

        fn def() -> App {
            App::new(Self::CMD).about(
                "Delete Anoma ledger node's and Tendermint node's storage \
                 data.",
            )
        }
    }

    #[derive(Clone, Debug)]
    pub enum Gossip {
        Run(GossipRun),
    }

    impl SubCmd for Gossip {
        const CMD: &'static str = "gossip";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).and_then(|matches| {
                let run = SubCmd::parse(matches).map(Gossip::Run);
                run
                    // The `run` command is the default if no sub-command given
                    .or_else(|| {
                        Some(Gossip::Run(GossipRun(args::GossipRun::parse(
                            matches,
                        ))))
                    })
            })
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about(
                    "Gossip node sub-commands. If no sub-command specified, \
                     defaults to run the node.",
                )
                .subcommand(GossipRun::def())
                .add_args::<args::GossipRun>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct GossipRun(pub args::GossipRun);

    impl SubCmd for GossipRun {
        const CMD: &'static str = "run";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| GossipRun(args::GossipRun::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Run a gossip node")
                .add_args::<args::GossipRun>()
        }
    }

    #[derive(Clone, Debug)]
    pub enum Config {
        Gen(ConfigGen),
    }

    impl SubCmd for Config {
        const CMD: &'static str = "config";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .and_then(|matches| SubCmd::parse(matches).map(Self::Gen))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .about("Configuration sub-commands")
                .subcommand(ConfigGen::def())
        }
    }

    #[derive(Clone, Debug)]
    pub struct ConfigGen;

    impl SubCmd for ConfigGen {
        const CMD: &'static str = "gen";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).map(|_matches| Self)
        }

        fn def() -> App {
            App::new(Self::CMD).about("Generate the default configuration file")
        }
    }

    #[derive(Clone, Debug)]
    pub struct TxCustom(pub args::TxCustom);

    impl SubCmd for TxCustom {
        const CMD: &'static str = "tx";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| TxCustom(args::TxCustom::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Send a transaction with custom WASM code")
                .add_args::<args::TxCustom>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct TxTransfer(pub args::TxTransfer);

    impl SubCmd for TxTransfer {
        const CMD: &'static str = "transfer";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| TxTransfer(args::TxTransfer::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Send a signed transfer transaction")
                .add_args::<args::TxTransfer>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct TxUpdateVp(pub args::TxUpdateVp);

    impl SubCmd for TxUpdateVp {
        const CMD: &'static str = "update";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| TxUpdateVp(args::TxUpdateVp::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about(
                    "Send a signed transaction to update account's validity \
                     predicate",
                )
                .add_args::<args::TxUpdateVp>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct TxInitAccount(pub args::TxInitAccount);

    impl SubCmd for TxInitAccount {
        const CMD: &'static str = "init-account";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).map(|matches| {
                TxInitAccount(args::TxInitAccount::parse(matches))
            })
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about(
                    "Send a signed transaction to create a new established \
                     account",
                )
                .add_args::<args::TxInitAccount>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct Bond(pub args::Bond);

    impl SubCmd for Bond {
        const CMD: &'static str = "bond";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| Bond(args::Bond::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Bond tokens in PoS system.")
                .add_args::<args::Bond>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct Unbond(pub args::Unbond);

    impl SubCmd for Unbond {
        const CMD: &'static str = "unbond";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| Unbond(args::Unbond::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Unbond tokens from a PoS bond.")
                .add_args::<args::Unbond>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct Withdraw(pub args::Withdraw);

    impl SubCmd for Withdraw {
        const CMD: &'static str = "withdraw";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| Withdraw(args::Withdraw::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Withdraw tokens from previously unbonded PoS bond.")
                .add_args::<args::Withdraw>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct QueryEpoch(pub args::Query);

    impl SubCmd for QueryEpoch {
        const CMD: &'static str = "epoch";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| QueryEpoch(args::Query::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Query the epoch of the last committed block")
                .add_args::<args::Query>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct QueryBalance(pub args::QueryBalance);

    impl SubCmd for QueryBalance {
        const CMD: &'static str = "balance";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| QueryBalance(args::QueryBalance::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Query balance(s) of tokens")
                .add_args::<args::QueryBalance>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct QueryBonds(pub args::QueryBonds);

    impl SubCmd for QueryBonds {
        const CMD: &'static str = "bonds";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| QueryBonds(args::QueryBonds::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Query PoS bond(s)")
                .add_args::<args::QueryBonds>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct QueryVotingPower(pub args::QueryVotingPower);

    impl SubCmd for QueryVotingPower {
        const CMD: &'static str = "voting-power";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).map(|matches| {
                QueryVotingPower(args::QueryVotingPower::parse(matches))
            })
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Query PoS voting power")
                .add_args::<args::QueryVotingPower>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct QuerySlashes(pub args::QuerySlashes);

    impl SubCmd for QuerySlashes {
        const CMD: &'static str = "slashes";

        fn parse(matches: &ArgMatches) -> Option<Self>
        where
            Self: Sized,
        {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| QuerySlashes(args::QuerySlashes::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Query PoS voting power")
                .add_args::<args::QuerySlashes>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct Intent(pub args::Intent);

    impl SubCmd for Intent {
        const CMD: &'static str = "intent";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches
                .subcommand_matches(Self::CMD)
                .map(|matches| Intent(args::Intent::parse(matches)))
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("Send an intent.")
                .add_args::<args::Intent>()
        }
    }

    #[derive(Clone, Debug)]
    pub struct SubscribeTopic(pub args::SubscribeTopic);

    impl SubCmd for SubscribeTopic {
        const CMD: &'static str = "subscribe-topic";

        fn parse(matches: &ArgMatches) -> Option<Self> {
            matches.subcommand_matches(Self::CMD).map(|matches| {
                SubscribeTopic(args::SubscribeTopic::parse(matches))
            })
        }

        fn def() -> App {
            App::new(Self::CMD)
                .about("subscribe to a topic.")
                .add_args::<args::SubscribeTopic>()
        }
    }
}

pub mod args {

    use std::convert::TryFrom;
    use std::fs::File;
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use std::str::FromStr;

    use anoma::types::address::Address;
    use anoma::types::intent::{DecimalWrapper, Exchange};
    use anoma::types::key::ed25519::PublicKey;
    use anoma::types::storage::Epoch;
    use anoma::types::token;
    use libp2p::Multiaddr;
    use serde::Deserialize;

    use super::context::{WalletAddress, WalletKeypair, WalletPublicKey};
    use super::utils::*;
    use super::ArgMatches;

    const ADDRESS: Arg<WalletAddress> = arg("address");
    const ALIAS_OPT: ArgOpt<String> = ALIAS.opt();
    const ALIAS: Arg<String> = arg("alias");
    const AMOUNT: Arg<token::Amount> = arg("amount");
    const BASE_DIR: ArgDefault<PathBuf> =
        arg_default("base-dir", DefaultFn(|| ".anoma".into()));
    const CODE_PATH: Arg<PathBuf> = arg("code-path");
    const CODE_PATH_OPT: ArgOpt<PathBuf> = CODE_PATH.opt();
    const DATA_PATH_OPT: ArgOpt<PathBuf> = arg_opt("data-path");
    const DATA_PATH: Arg<PathBuf> = arg("data-path");
    const DECRYPT: ArgFlag = flag("decrypt");
    const DRY_RUN_TX: ArgFlag = flag("dry-run");
    const EPOCH: ArgOpt<Epoch> = arg_opt("epoch");
    const FILTER_PATH: ArgOpt<PathBuf> = arg_opt("filter-path");
    const LEDGER_ADDRESS_ABOUT: &str =
        "Address of a ledger node as \"{scheme}://{host}:{port}\". If the \
         scheme is not supplied, it is assumed to be TCP.";
    const LEDGER_ADDRESS_DEFAULT: ArgDefault<tendermint::net::Address> =
        LEDGER_ADDRESS.default(DefaultFn(|| {
            let raw = "127.0.0.1:26657";
            tendermint::net::Address::from_str(raw).unwrap()
        }));
    const LEDGER_ADDRESS_OPT: ArgOpt<tendermint::net::Address> =
        LEDGER_ADDRESS.opt();
    const LEDGER_ADDRESS: Arg<tendermint::net::Address> = arg("ledger-address");
    const MATCHMAKER_PATH: ArgOpt<PathBuf> = arg_opt("matchmaker-path");
    const MULTIADDR_OPT: ArgOpt<Multiaddr> = arg_opt("address");
    const NODE_OPT: ArgOpt<String> = arg_opt("node");
    const NODE: Arg<String> = arg("node");
    const OWNER: ArgOpt<WalletAddress> = arg_opt("owner");
    const PEERS: ArgMulti<String> = arg_multi("peers");
    const PUBLIC_KEY: Arg<WalletPublicKey> = arg("public-key");
    const RAW_ADDRESS: Arg<Address> = arg("address");
    const RAW_PUBLIC_KEY_OPT: ArgOpt<PublicKey> = arg_opt("public-key");
    const RPC_SOCKET_ADDR: ArgOpt<SocketAddr> = arg_opt("rpc");
    const SIGNER: ArgOpt<WalletAddress> = arg_opt("signer");
    const SIGNING_KEY_OPT: ArgOpt<WalletKeypair> = SIGNING_KEY.opt();
    const SIGNING_KEY: Arg<WalletKeypair> = arg("signing-key");
    const SOURCE: Arg<WalletAddress> = arg("source");
    const SOURCE_OPT: ArgOpt<WalletAddress> = SOURCE.opt();
    const TARGET: Arg<WalletAddress> = arg("target");
    const TO_STDOUT: ArgFlag = flag("stdout");
    const TOKEN_OPT: ArgOpt<WalletAddress> = TOKEN.opt();
    const TOKEN: Arg<WalletAddress> = arg("token");
    const TOPIC_OPT: ArgOpt<String> = arg_opt("topic");
    const TOPIC: Arg<String> = arg("topic");
    const TOPICS: ArgMulti<String> = TOPIC.multi();
    const TX_CODE_PATH: ArgOpt<PathBuf> = arg_opt("tx-code-path");
    const UNSAFE_DONT_ENCRYPT: ArgFlag = flag("unsafe-dont-encrypt");
    const UNSAFE_SHOW_SECRET: ArgFlag = flag("unsafe-show-secret");
    const VALIDATOR: Arg<WalletAddress> = arg("validator");
    const VALIDATOR_OPT: ArgOpt<WalletAddress> = VALIDATOR.opt();
    const VALUE: ArgOpt<String> = arg_opt("value");

    /// Global command arguments
    #[derive(Clone, Debug)]
    pub struct Global {
        pub base_dir: PathBuf,
    }

    impl Global {
        /// Parse global arguments
        pub fn parse(matches: &ArgMatches) -> Self {
            let base_dir = BASE_DIR.parse(matches);
            Global { base_dir }
        }

        /// Add global args definition. Should be added to every top-level
        /// command.
        pub fn def(app: App) -> App {
            app.arg(BASE_DIR.def().about(
                "The base directory is where the client and nodes \
                 configuration and state is stored.",
            ))
        }
    }

    /// Custom transaction arguments
    #[derive(Clone, Debug)]
    pub struct TxCustom {
        /// Common tx arguments
        pub tx: Tx,
        /// Path to the tx WASM code file
        pub code_path: PathBuf,
        /// Path to the data file
        pub data_path: Option<PathBuf>,
        /// Sign the tx with the key for the given alias from your wallet
        pub signing_key: Option<WalletKeypair>,
        /// Sign the tx with the keypair of the public key of the given address
        pub signer: Option<WalletAddress>,
    }

    impl Args for TxCustom {
        fn parse(matches: &ArgMatches) -> Self {
            let tx = Tx::parse(matches);
            let code_path = CODE_PATH.parse(matches);
            let data_path = DATA_PATH_OPT.parse(matches);
            let signing_key = SIGNING_KEY_OPT.parse(matches);
            let signer = SIGNER.parse(matches);
            Self {
                tx,
                code_path,
                data_path,
                signing_key,
                signer,
            }
        }

        fn def(app: App) -> App {
            app.add_args::<Tx>()
                .arg(
                    CODE_PATH
                        .def()
                        .about("The path to the transaction's WASM code."),
                )
                .arg(DATA_PATH_OPT.def().about(
                    "The data file at this path containing arbitrary bytes \
                     will be passed to the transaction code when it's \
                     executed.",
                ))
                .arg(
                    SIGNING_KEY_OPT
                        .def()
                        .about(
                            "Sign the transaction with the key for the given \
                             public key, public key hash or alias from your \
                             wallet.",
                        )
                        .conflicts_with(SIGNER.name),
                )
                .arg(
                    SIGNER
                        .def()
                        .about(
                            "Sign the transaction with the keypair of the \
                             public key of the given address.",
                        )
                        .conflicts_with(SIGNING_KEY_OPT.name),
                )
        }
    }

    /// Transfer transaction arguments
    #[derive(Clone, Debug)]
    pub struct TxTransfer {
        /// Common tx arguments
        pub tx: Tx,
        /// Transfer source address
        pub source: WalletAddress,
        /// Transfer target address
        pub target: WalletAddress,
        /// Transferred token address
        pub token: WalletAddress,
        /// Transferred token amount
        pub amount: token::Amount,
    }

    impl Args for TxTransfer {
        fn parse(matches: &ArgMatches) -> Self {
            let tx = Tx::parse(matches);
            let source = SOURCE.parse(matches);
            let target = TARGET.parse(matches);
            let token = TOKEN.parse(matches);
            let amount = AMOUNT.parse(matches);
            Self {
                tx,
                source,
                target,
                token,
                amount,
            }
        }

        fn def(app: App) -> App {
            app.add_args::<Tx>()
                .arg(SOURCE.def().about(
                    "The source account address. The source's key is used to \
                     produce the signature.",
                ))
                .arg(TARGET.def().about("The target account address."))
                .arg(TOKEN.def().about("The transfer token."))
                .arg(AMOUNT.def().about("The amount to transfer in decimal."))
        }
    }

    /// Transaction to initialize a new account
    #[derive(Clone, Debug)]
    pub struct TxInitAccount {
        /// Common tx arguments
        pub tx: Tx,
        /// Address of the source account
        pub source: WalletAddress,
        /// Path to the VP WASM code file for the new account
        pub vp_code_path: Option<PathBuf>,
        /// Public key for the new account
        pub public_key: WalletPublicKey,
    }

    impl Args for TxInitAccount {
        fn parse(matches: &ArgMatches) -> Self {
            let tx = Tx::parse(matches);
            let source = SOURCE.parse(matches);
            let vp_code_path = CODE_PATH_OPT.parse(matches);
            let public_key = PUBLIC_KEY.parse(matches);
            Self {
                tx,
                source,
                vp_code_path,
                public_key,
            }
        }

        fn def(app: App) -> App {
            app.add_args::<Tx>()
                .arg(SOURCE.def().about(
                    "The source account's address that signs the transaction.",
                ))
                .arg(CODE_PATH_OPT.def().about(
                    "The path to the validity predicate WASM code to be used \
                     for the new account. Uses the default user VP if none \
                     specified.",
                ))
                .arg(PUBLIC_KEY.def().about(
                    "A public key to be used for the new account in \
                     hexadecimal encoding.",
                ))
        }
    }

    /// Transaction to update a VP arguments
    #[derive(Clone, Debug)]
    pub struct TxUpdateVp {
        /// Common tx arguments
        pub tx: Tx,
        /// Path to the VP WASM code file
        pub vp_code_path: PathBuf,
        /// Address of the account whose VP is to be updated
        pub addr: WalletAddress,
    }

    impl Args for TxUpdateVp {
        fn parse(matches: &ArgMatches) -> Self {
            let tx = Tx::parse(matches);
            let vp_code_path = CODE_PATH.parse(matches);
            let addr = ADDRESS.parse(matches);
            Self {
                tx,
                vp_code_path,
                addr,
            }
        }

        fn def(app: App) -> App {
            app.add_args::<Tx>()
                .arg(
                    CODE_PATH.def().about(
                        "The path to the new validity predicate WASM code.",
                    ),
                )
                .arg(ADDRESS.def().about(
                    "The account's address. It's key is used to produce the \
                     signature.",
                ))
        }
    }

    /// Bond arguments
    #[derive(Clone, Debug)]
    pub struct Bond {
        /// Common tx arguments
        pub tx: Tx,
        /// Validator address
        pub validator: WalletAddress,
        /// Amount of tokens to stake in a bond
        pub amount: token::Amount,
        /// Source address for delegations. For self-bonds, the validator is
        /// also the source.
        pub source: Option<WalletAddress>,
    }

    impl Args for Bond {
        fn parse(matches: &ArgMatches) -> Self {
            let tx = Tx::parse(matches);
            let validator = VALIDATOR.parse(matches);
            let amount = AMOUNT.parse(matches);
            let source = SOURCE_OPT.parse(matches);
            Self {
                tx,
                validator,
                amount,
                source,
            }
        }

        fn def(app: App) -> App {
            app.add_args::<Tx>()
                .arg(VALIDATOR.def().about("Validator address."))
                .arg(AMOUNT.def().about("Amount of tokens to stake in a bond."))
                .arg(SOURCE_OPT.def().about(
                    "Source address for delegations. For self-bonds, the \
                     validator is also the source",
                ))
        }
    }

    /// Unbond arguments
    #[derive(Clone, Debug)]
    pub struct Unbond {
        /// Common tx arguments
        pub tx: Tx,
        /// Validator address
        pub validator: WalletAddress,
        /// Amount of tokens to unbond from a bond
        pub amount: token::Amount,
        /// Source address for unbonding from delegations. For unbonding from
        /// self-bonds, the validator is also the source
        pub source: Option<WalletAddress>,
    }

    impl Args for Unbond {
        fn parse(matches: &ArgMatches) -> Self {
            let tx = Tx::parse(matches);
            let validator = VALIDATOR.parse(matches);
            let amount = AMOUNT.parse(matches);
            let source = SOURCE_OPT.parse(matches);
            Self {
                tx,
                validator,
                amount,
                source,
            }
        }

        fn def(app: App) -> App {
            app.add_args::<Tx>()
                .arg(VALIDATOR.def().about("Validator address."))
                .arg(
                    AMOUNT
                        .def()
                        .about("Amount of tokens to unbond from a bond."),
                )
                .arg(SOURCE_OPT.def().about(
                    "Source address for unbonding from delegations. For \
                     unbonding from self-bonds, the validator is also the \
                     source",
                ))
        }
    }

    /// Withdraw arguments
    #[derive(Clone, Debug)]
    pub struct Withdraw {
        /// Common tx arguments
        pub tx: Tx,
        /// Validator address
        pub validator: WalletAddress,
        /// Source address for withdrawing from delegations. For withdrawing
        /// from self-bonds, the validator is also the source
        pub source: Option<WalletAddress>,
    }

    impl Args for Withdraw {
        fn parse(matches: &ArgMatches) -> Self {
            let tx = Tx::parse(matches);
            let validator = VALIDATOR.parse(matches);
            let source = SOURCE_OPT.parse(matches);
            Self {
                tx,
                validator,
                source,
            }
        }

        fn def(app: App) -> App {
            app.add_args::<Tx>()
                .arg(VALIDATOR.def().about("Validator address."))
                .arg(SOURCE_OPT.def().about(
                    "Source address for withdrawing from delegations. For \
                     withdrawing from self-bonds, the validator is also the \
                     source",
                ))
        }
    }

    /// Query token balance(s)
    #[derive(Clone, Debug)]
    pub struct QueryBalance {
        /// Common query args
        pub query: Query,
        /// Address of an owner
        pub owner: Option<WalletAddress>,
        /// Address of a token
        pub token: Option<WalletAddress>,
    }

    impl Args for QueryBalance {
        fn parse(matches: &ArgMatches) -> Self {
            let query = Query::parse(matches);
            let owner = OWNER.parse(matches);
            let token = TOKEN_OPT.parse(matches);
            Self {
                query,
                owner,
                token,
            }
        }

        fn def(app: App) -> App {
            app.add_args::<Query>()
                .arg(
                    OWNER
                        .def()
                        .about("The account address whose balance to query"),
                )
                .arg(
                    TOKEN_OPT
                        .def()
                        .about("The token's address whose balance to query"),
                )
        }
    }

    /// Helper struct for generating intents
    #[derive(Debug, Clone, Deserialize)]
    pub struct ExchangeDefinition {
        /// The source address
        pub addr: String,
        /// The token to be sold
        pub token_sell: String,
        /// The minimum rate
        pub rate_min: String,
        /// The maximum amount of token to be sold
        pub max_sell: String,
        /// The token to be bought
        pub token_buy: String,
        /// The amount of token to be bought
        pub min_buy: String,
        // The path to the wasm vp code
        pub vp_path: Option<String>,
    }

    impl TryFrom<ExchangeDefinition> for Exchange {
        type Error = &'static str;

        fn try_from(
            value: ExchangeDefinition,
        ) -> Result<Exchange, Self::Error> {
            let vp = if let Some(path) = value.vp_path {
                if let Ok(wasm) = std::fs::read(path.clone()) {
                    Some(wasm)
                } else {
                    eprintln!("File {} was not found.", path);
                    None
                }
            } else {
                None
            };

            let addr = Address::decode(value.addr)
                .expect("Addr should be a valid address");
            let token_buy = Address::decode(value.token_buy)
                .expect("Token_buy should be a valid address");
            let token_sell = Address::decode(value.token_sell)
                .expect("Token_sell should be a valid address");
            let min_buy = token::Amount::from_str(&value.min_buy)
                .expect("Min_buy must be convertible to number");
            let max_sell = token::Amount::from_str(&value.max_sell)
                .expect("Max_sell must be convertible to number");
            let rate_min = DecimalWrapper::from_str(&value.rate_min)
                .expect("Max_sell must be convertible to decimal.");

            Ok(Exchange {
                addr,
                token_sell,
                rate_min,
                max_sell,
                token_buy,
                min_buy,
                vp,
            })
        }
    }

    /// Query PoS bond(s)
    #[derive(Clone, Debug)]
    pub struct QueryBonds {
        /// Common query args
        pub query: Query,
        /// Address of an owner
        pub owner: Option<WalletAddress>,
        /// Address of a validator
        pub validator: Option<WalletAddress>,
    }

    impl Args for QueryBonds {
        fn parse(matches: &ArgMatches) -> Self {
            let query = Query::parse(matches);
            let owner = OWNER.parse(matches);
            let validator = VALIDATOR_OPT.parse(matches);
            Self {
                query,
                owner,
                validator,
            }
        }

        fn def(app: App) -> App {
            app.add_args::<Query>()
                .arg(
                    OWNER.def().about(
                        "The owner account address whose bonds to query",
                    ),
                )
                .arg(
                    VALIDATOR_OPT
                        .def()
                        .about("The validator's address whose bonds to query"),
                )
        }
    }

    /// Query PoS voting power
    #[derive(Clone, Debug)]
    pub struct QueryVotingPower {
        /// Common query args
        pub query: Query,
        /// Address of a validator
        pub validator: Option<WalletAddress>,
        /// Epoch in which to find voting power
        pub epoch: Option<Epoch>,
    }

    impl Args for QueryVotingPower {
        fn parse(matches: &ArgMatches) -> Self {
            let query = Query::parse(matches);
            let validator = VALIDATOR_OPT.parse(matches);
            let epoch = EPOCH.parse(matches);
            Self {
                query,
                validator,
                epoch,
            }
        }

        fn def(app: App) -> App {
            app.add_args::<Query>()
                .arg(VALIDATOR_OPT.def().about(
                    "The validator's address whose voting power to query",
                ))
                .arg(EPOCH.def().about(
                    "The epoch at which to query (last committed, if not \
                     specified)",
                ))
        }
    }

    /// Query PoS slashes
    #[derive(Clone, Debug)]
    pub struct QuerySlashes {
        /// Common query args
        pub query: Query,
        /// Address of a validator
        pub validator: Option<WalletAddress>,
    }

    impl Args for QuerySlashes {
        fn parse(matches: &ArgMatches) -> Self {
            let query = Query::parse(matches);
            let validator = VALIDATOR_OPT.parse(matches);
            Self { query, validator }
        }

        fn def(app: App) -> App {
            app.add_args::<Query>().arg(
                VALIDATOR_OPT
                    .def()
                    .about("The validator's address whose slashes to query"),
            )
        }
    }

    /// Intent arguments
    #[derive(Clone, Debug)]
    pub struct Intent {
        /// Gossip node address
        pub node_addr: Option<String>,
        /// Intent topic
        pub topic: Option<String>,
        /// Signing key
        pub signing_key: WalletKeypair,
        /// Exchanges description
        pub exchanges: Vec<Exchange>,
        /// The address of the ledger node as host:port
        pub ledger_address: tendermint::net::Address,
        /// Print output to stdout
        pub to_stdout: bool,
    }

    impl Args for Intent {
        fn parse(matches: &ArgMatches) -> Self {
            let signing_key = SIGNING_KEY.parse(matches);
            let node_addr = NODE_OPT.parse(matches);
            let data_path = DATA_PATH.parse(matches);
            let to_stdout = TO_STDOUT.parse(matches);
            let topic = TOPIC_OPT.parse(matches);

            let file = File::open(&data_path).expect("File must exist.");
            let exchange_definitions: Vec<ExchangeDefinition> =
                serde_json::from_reader(file)
                    .expect("JSON was not well-formatted");

            let exchanges: Vec<Exchange> = exchange_definitions
                .iter()
                .map(|item| {
                    Exchange::try_from(item.clone()).expect(
                        "Conversion from ExchangeDefinition to Exchange \
                         should not fail.",
                    )
                })
                .collect();
            let ledger_address = LEDGER_ADDRESS_DEFAULT.parse(matches);

            Self {
                node_addr,
                topic,
                signing_key,
                exchanges,
                ledger_address,
                to_stdout,
            }
        }

        fn def(app: App) -> App {
            app.arg(
                NODE_OPT
                    .def()
                    .about("The gossip node address.")
                    .conflicts_with(TO_STDOUT.name),
            )
            .arg(SIGNING_KEY.def().about(
                "Sign the intent with the key for the given public key, \
                 public key hash or alias from your wallet.",
            ))
            .arg(DATA_PATH.def().about(
                "The data of the intent, that contains all value necessary \
                 for the matchmaker.",
            ))
            .arg(LEDGER_ADDRESS_DEFAULT.def().about(LEDGER_ADDRESS_ABOUT))
            .arg(
                TOPIC_OPT
                    .def()
                    .about("The subnetwork where the intent should be sent to")
                    .conflicts_with(TO_STDOUT.name),
            )
            .arg(
                TO_STDOUT
                    .def()
                    .about(
                        "Echo the serialized intent to stdout. Note that with \
                         this option, the intent won't be submitted to the \
                         intent gossiper RPC.",
                    )
                    .conflicts_with_all(&[NODE_OPT.name, TOPIC.name]),
            )
        }
    }

    /// Subscribe intent topic arguments
    #[derive(Clone, Debug)]
    pub struct SubscribeTopic {
        /// Gossip node address
        pub node_addr: String,
        /// Intent topic
        pub topic: String,
    }

    impl Args for SubscribeTopic {
        fn parse(matches: &ArgMatches) -> Self {
            let node_addr = NODE.parse(matches);
            let topic = TOPIC.parse(matches);
            Self { node_addr, topic }
        }

        fn def(app: App) -> App {
            app.arg(NODE.def().about("The gossip node address.")).arg(
                TOPIC
                    .def()
                    .about("The new topic of interest for that node."),
            )
        }
    }

    #[derive(Clone, Debug)]
    pub struct GossipRun {
        pub addr: Option<Multiaddr>,
        pub peers: Vec<String>,
        pub topics: Vec<String>,
        pub rpc: Option<SocketAddr>,
        pub matchmaker_path: Option<PathBuf>,
        pub tx_code_path: Option<PathBuf>,
        pub ledger_addr: Option<tendermint::net::Address>,
        pub filter_path: Option<PathBuf>,
    }

    impl Args for GossipRun {
        fn parse(matches: &ArgMatches) -> Self {
            let addr = MULTIADDR_OPT.parse(matches);
            let peers = PEERS.parse(matches);
            let topics = TOPICS.parse(matches);
            let rpc = RPC_SOCKET_ADDR.parse(matches);
            let matchmaker_path = MATCHMAKER_PATH.parse(matches);
            let tx_code_path = TX_CODE_PATH.parse(matches);
            let ledger_addr = LEDGER_ADDRESS_OPT.parse(matches);
            let filter_path = FILTER_PATH.parse(matches);
            Self {
                addr,
                peers,
                topics,
                rpc,
                matchmaker_path,
                tx_code_path,
                ledger_addr,
                filter_path,
            }
        }

        fn def(app: App) -> App {
            app.arg(
                MULTIADDR_OPT
                    .def()
                    .about("Gossip service address as host:port."),
            )
            .arg(PEERS.def().about("List of peers to connect to."))
            .arg(TOPICS.def().about("Enable a new gossip topic."))
            .arg(RPC_SOCKET_ADDR.def().about("Enable RPC service."))
            .arg(MATCHMAKER_PATH.def().about("The matchmaker."))
            .arg(
                TX_CODE_PATH
                    .def()
                    .about("The transaction code to use with the matchmaker"),
            )
            .arg(LEDGER_ADDRESS_OPT.def().about(
                "The address of the ledger as \"{scheme}://{host}:{port}\" \
                 that the matchmaker must send transactions to. If the scheme \
                 is not supplied, it is assumed to be TCP.",
            ))
            .arg(
                FILTER_PATH
                    .def()
                    .about("The private filter for the matchmaker"),
            )
        }
    }

    /// Common transaction arguments
    #[derive(Clone, Debug)]
    pub struct Tx {
        /// Simulate applying the transaction
        pub dry_run: bool,
        /// The address of the ledger node as host:port
        pub ledger_address: tendermint::net::Address,
        /// If any new account is initialized by the tx, use the given alias to
        /// save it in the wallet.
        pub initialized_account_alias: Option<String>,
    }

    impl Args for Tx {
        fn def(app: App) -> App {
            app.arg(
                DRY_RUN_TX
                    .def()
                    .about("Simulate the transaction application."),
            )
            .arg(LEDGER_ADDRESS_DEFAULT.def().about(LEDGER_ADDRESS_ABOUT))
            .arg(ALIAS_OPT.def().about(
                "If any new account is initialized by the tx, use the given \
                 alias to save it in the wallet. If multiple accounts are \
                 initialized, the alias will be the prefix of each new \
                 address joined with a number",
            ))
        }

        fn parse(matches: &ArgMatches) -> Self {
            let dry_run = DRY_RUN_TX.parse(matches);
            let ledger_address = LEDGER_ADDRESS_DEFAULT.parse(matches);
            let initialized_account_alias = ALIAS_OPT.parse(matches);
            Self {
                dry_run,
                ledger_address,
                initialized_account_alias,
            }
        }
    }

    /// Common query arguments
    #[derive(Clone, Debug)]
    pub struct Query {
        /// The address of the ledger node as host:port
        pub ledger_address: tendermint::net::Address,
    }

    impl Args for Query {
        fn def(app: App) -> App {
            app.arg(LEDGER_ADDRESS_DEFAULT.def().about(LEDGER_ADDRESS_ABOUT))
        }

        fn parse(matches: &ArgMatches) -> Self {
            let ledger_address = LEDGER_ADDRESS_DEFAULT.parse(matches);
            Self { ledger_address }
        }
    }

    /// Wallet generate key and implicit address arguments
    #[derive(Clone, Debug)]
    pub struct KeyAndAddressGen {
        /// Key alias
        pub alias: Option<String>,
        /// Don't encrypt the keypair
        pub unsafe_dont_encrypt: bool,
    }

    impl Args for KeyAndAddressGen {
        fn parse(matches: &ArgMatches) -> Self {
            let alias = ALIAS_OPT.parse(matches);
            let unsafe_dont_encrypt = UNSAFE_DONT_ENCRYPT.parse(matches);
            Self {
                alias,
                unsafe_dont_encrypt,
            }
        }

        fn def(app: App) -> App {
            app.arg(ALIAS_OPT.def().about(
                "The key and address alias. If none provided, the alias will \
                 be the public key hash.",
            ))
            .arg(UNSAFE_DONT_ENCRYPT.def().about(
                "UNSAFE: Do not encrypt the keypair. Do not use this for keys \
                 used in a live network.",
            ))
        }
    }

    /// Wallet key lookup arguments
    #[derive(Clone, Debug)]
    pub struct KeyFind {
        pub public_key: Option<PublicKey>,
        pub alias: Option<String>,
        pub value: Option<String>,
        pub unsafe_show_secret: bool,
    }

    impl Args for KeyFind {
        fn parse(matches: &ArgMatches) -> Self {
            let public_key = RAW_PUBLIC_KEY_OPT.parse(matches);
            let alias = ALIAS_OPT.parse(matches);
            let value = VALUE.parse(matches);
            let unsafe_show_secret = UNSAFE_SHOW_SECRET.parse(matches);

            Self {
                public_key,
                alias,
                value,
                unsafe_show_secret,
            }
        }

        fn def(app: App) -> App {
            app.arg(
                RAW_PUBLIC_KEY_OPT
                    .def()
                    .about("A public key associated with the keypair")
                    .conflicts_with("alias")
                    .conflicts_with("value"),
            )
            .arg(
                ALIAS_OPT
                    .def()
                    .about("An alias associated with the keypair")
                    .conflicts_with("value"),
            )
            .arg(
                VALUE
                    .def()
                    .about("A public key or alias associated with the keypair"),
            )
            .arg(
                UNSAFE_SHOW_SECRET
                    .def()
                    .about("UNSAFE: Print the secret key"),
            )
        }
    }

    /// Wallet list keys arguments
    #[derive(Clone, Debug)]
    pub struct KeyList {
        pub decrypt: bool,
        pub unsafe_show_secret: bool,
    }

    impl Args for KeyList {
        fn parse(matches: &ArgMatches) -> Self {
            let decrypt = DECRYPT.parse(matches);
            let unsafe_show_secret = UNSAFE_SHOW_SECRET.parse(matches);
            Self {
                decrypt,
                unsafe_show_secret,
            }
        }

        fn def(app: App) -> App {
            app.arg(DECRYPT.def().about("Decrypt keys that are encrypted"))
                .arg(
                    UNSAFE_SHOW_SECRET
                        .def()
                        .about("UNSAFE: Print the secret keys"),
                )
        }
    }

    /// Wallet key export arguments
    #[derive(Clone, Debug)]
    pub struct KeyExport {
        pub alias: String,
    }

    impl Args for KeyExport {
        fn parse(matches: &ArgMatches) -> Self {
            let alias = ALIAS.parse(matches);

            Self { alias }
        }

        fn def(app: App) -> App {
            app.arg(
                ALIAS.def().about("The alias of the key you wish to export"),
            )
        }
    }

    /// Wallet address lookup arguments
    #[derive(Clone, Debug)]
    pub struct AddressFind {
        pub alias: String,
    }

    impl Args for AddressFind {
        fn parse(matches: &ArgMatches) -> Self {
            let alias = ALIAS.parse(matches);
            Self { alias }
        }

        fn def(app: App) -> App {
            app.arg(
                ALIAS_OPT
                    .def()
                    .about("An alias associated with the address"),
            )
        }
    }

    /// Wallet address add arguments
    #[derive(Clone, Debug)]
    pub struct AddressAdd {
        pub alias: String,
        pub address: Address,
    }

    impl Args for AddressAdd {
        fn parse(matches: &ArgMatches) -> Self {
            let alias = ALIAS.parse(matches);
            let address = RAW_ADDRESS.parse(matches);
            Self { alias, address }
        }

        fn def(app: App) -> App {
            app.arg(
                ALIAS
                    .def()
                    .about("An alias to be associated with the address"),
            )
            .arg(
                RAW_ADDRESS
                    .def()
                    .about("The bech32m encoded address string"),
            )
        }
    }
}
pub fn anoma_cli() -> (cmds::Anoma, String) {
    let app = anoma_app();
    let matches = app.get_matches();
    let raw_sub_cmd =
        matches.subcommand().map(|(raw, _matches)| raw.to_string());
    let result = cmds::Anoma::parse(&matches);
    match (result, raw_sub_cmd) {
        (Some(cmd), Some(raw_sub)) => return (cmd, raw_sub),
        _ => {
            anoma_app().print_help().unwrap();
        }
    }
    safe_exit(2);
}

pub fn anoma_node_cli() -> (cmds::AnomaNode, Context) {
    let app = anoma_node_app();
    cmds::AnomaNode::parse_or_print_help(app)
}

pub fn anoma_client_cli() -> (cmds::AnomaClient, Context) {
    let app = anoma_client_app();
    cmds::AnomaClient::parse_or_print_help(app)
}

pub fn anoma_wallet_cli() -> (cmds::AnomaWallet, Context) {
    let app = anoma_wallet_app();
    cmds::AnomaWallet::parse_or_print_help(app)
}

fn anoma_app() -> App {
    let app = App::new(APP_NAME)
        .version(CLI_VERSION)
        .author(AUTHOR)
        .about("Anoma command line interface.")
        .setting(AppSettings::SubcommandRequiredElseHelp);
    cmds::Anoma::add_sub(args::Global::def(app))
}

fn anoma_node_app() -> App {
    let app = App::new(APP_NAME)
        .version(NODE_VERSION)
        .author(AUTHOR)
        .about("Anoma node command line interface.")
        .setting(AppSettings::SubcommandRequiredElseHelp);
    cmds::AnomaNode::add_sub(args::Global::def(app))
}

fn anoma_client_app() -> App {
    let app = App::new(APP_NAME)
        .version(CLIENT_VERSION)
        .author(AUTHOR)
        .about("Anoma client command line interface.")
        .setting(AppSettings::SubcommandRequiredElseHelp);
    cmds::AnomaClient::add_sub(args::Global::def(app))
}

fn anoma_wallet_app() -> App {
    let app = App::new(APP_NAME)
        .version(WALLET_VERSION)
        .author(AUTHOR)
        .about("Anoma wallet command line interface.")
        .setting(AppSettings::SubcommandRequiredElseHelp);
    cmds::AnomaWallet::add_sub(args::Global::def(app))
}

pub fn update_gossip_config(
    args: args::GossipRun,
    config: &mut config::IntentGossiper,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(addr) = args.addr {
        config.address = addr
    }

    let matchmaker_arg = args.matchmaker_path;
    let tx_code_arg = args.tx_code_path;
    let ledger_address_arg = args.ledger_addr;
    let filter_arg = args.filter_path;
    if let Some(mut matchmaker_cfg) = config.matchmaker.as_mut() {
        if let Some(matchmaker) = matchmaker_arg {
            matchmaker_cfg.matchmaker = matchmaker
        }
        if let Some(tx_code) = tx_code_arg {
            matchmaker_cfg.tx_code = tx_code
        }
        if let Some(ledger_address) = ledger_address_arg {
            matchmaker_cfg.ledger_address = ledger_address
        }
        if let Some(filter) = filter_arg {
            matchmaker_cfg.filter = Some(filter)
        }
    } else if let (Some(matchmaker), Some(tx_code), Some(ledger_address)) = (
        matchmaker_arg.as_ref(),
        tx_code_arg.as_ref(),
        ledger_address_arg.as_ref(),
    ) {
        let matchmaker_cfg = Some(config::Matchmaker {
            matchmaker: matchmaker.clone(),
            tx_code: tx_code.clone(),
            ledger_address: ledger_address.clone(),
            filter: filter_arg,
        });
        config.matchmaker = matchmaker_cfg
    } else if matchmaker_arg.is_some()
        || tx_code_arg.is_some()
        || ledger_address_arg.is_some()
    // if at least one argument is not none then fail
    {
        panic!(
            "No complete matchmaker configuration found (matchmaker code \
             path, tx code path, and ledger address). Please update the \
             configuration with default value or use all cli argument to use \
             the matchmaker"
        );
    }
    if let Some(address) = args.rpc {
        config.rpc = Some(config::RpcServer { address });
    }
    Ok(())
}
