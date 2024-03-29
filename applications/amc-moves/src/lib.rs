//! An example of implementing an application to be tested with AMC.
//!
//! Run with `cargo run --release --bin amc-moves -- --help`
//!
//! This models concurrently moving elements in a list.
//!
//! From https://dl.acm.org/doi/pdf/10.1145/3380787.3393677

use amc::application::Application;
use amc::application::DerefDocument;
use amc::application::Document;
use amc::driver::ApplicationMsg;
use amc::driver::Drive;
use amc::global::GlobalActor;
use amc::global::GlobalActorState;
use amc::global::GlobalMsg;
use amc::model::ModelBuilder;
use amc::properties::syncing_done;
use automerge::transaction::Transactable;
use automerge::ObjType;
use automerge::ReadDoc;
use automerge::ROOT;
use stateright::actor::ActorModel;
use stateright::Property;
use std::borrow::Cow;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct List {
    pub initial_size: usize,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct ListState {
    pub doc: Document,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum ListInput {
    /// Move element from index to index.
    Move(usize, usize),
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
enum ListOutput {}

impl Application for List {
    type Input = ListInput;
    type Output = ();
    type State = ListState;

    fn init(&self, id: usize) -> Self::State {
        let mut doc = Document::new(id);
        doc.with_initial_change(|txn| {
            let list_id = txn.put_object(ROOT, "list", ObjType::List).unwrap();

            // for now start with 3 elements

            for i in 0..self.initial_size {
                txn.insert(
                    &list_id,
                    i,
                    char::from_u32(('a' as u32) + i as u32).unwrap(),
                )
                .unwrap();
            }
        });

        ListState { doc }
    }

    fn execute(&self, state: &mut Cow<Self::State>, input: Self::Input) -> Option<()> {
        match input {
            ListInput::Move(from, mut to) => {
                let (_, list_id) = state.document().get(ROOT, "list").unwrap().unwrap();
                let mut txn = state.to_mut().document_mut().transaction();
                let (item, _) = txn.get(&list_id, from).unwrap().unwrap();
                let item = item.into_scalar().unwrap();
                txn.delete(&list_id, from).unwrap();
                if from < to {
                    to -= 1;
                }
                txn.insert(&list_id, to, item).unwrap();
                txn.commit();
            }
        }
        None
    }
}

impl DerefDocument for ListState {
    fn document(&self) -> &Document {
        &self.doc
    }
    fn document_mut(&mut self) -> &mut Document {
        &mut self.doc
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Driver {
    pub func: DriverFunc,
    pub initial_size: usize,
}

/// Action for the application to perform.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum DriverFunc {
    MoverLastFirst,
    MoverFirstLast,
}

impl Drive<List> for Driver {
    type State = ();

    fn init(
        &self,
        _application_id: usize,
    ) -> (
        <Self as Drive<List>>::State,
        Vec<<List as Application>::Input>,
    ) {
        match self.func {
            DriverFunc::MoverLastFirst => {
                let msgs = vec![ListInput::Move(self.initial_size - 1, 0)];
                ((), msgs)
            }
            DriverFunc::MoverFirstLast => {
                let msgs = vec![ListInput::Move(0, self.initial_size - 1)];
                ((), msgs)
            }
        }
    }

    fn handle_output(
        &self,
        _state: &mut Cow<Self::State>,
        _output: <List as Application>::Output,
    ) -> Vec<<List as Application>::Input> {
        Vec::new()
    }
}

#[derive(clap::Args, Debug)]
pub struct MovesOpts {
    /// Initial size of the list.
    #[clap(long, global = true, default_value = "2")]
    pub initial_size: usize,
}

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    pub moves_opts: MovesOpts,

    #[clap(flatten)]
    pub amc_args: amc::cli::RunArgs,
}

impl ModelBuilder for MovesOpts {
    type App = List;

    type Driver = Driver;

    type Config = Config;

    type History = Vec<GlobalMsg<List>>;

    fn application(&self, _application: usize, _config: &Config) -> Self::App {
        List {
            initial_size: self.initial_size,
        }
    }

    fn drivers(&self, _application: usize, _config: &Config) -> Vec<Self::Driver> {
        vec![
            Driver {
                func: DriverFunc::MoverLastFirst,
                initial_size: self.initial_size,
            },
            Driver {
                func: DriverFunc::MoverFirstLast,
                initial_size: self.initial_size,
            },
        ]
    }

    fn config(&self, _model_opts: &amc::model::ModelOpts) -> Self::Config {
        Config {}
    }

    fn history(&self) -> Self::History {
        Vec::new()
    }

    fn properties(
        &self,
    ) -> Vec<
        stateright::Property<
            ActorModel<GlobalActor<Self::App, Self::Driver>, Self::Config, Self::History>,
        >,
    > {
        type Prop = Property<ActorModel<GlobalActor<List, Driver>, Config, Vec<GlobalMsg<List>>>>;
        vec![Prop::always(
            "no duplicates when in sync",
            |_model, state| {
                // When states are in sync, there shouldn't be any duplicate entries
                if !syncing_done(state) {
                    return true;
                }

                state.actor_states.iter().all(|s| {
                    if let GlobalActorState::Server(n) = &**s {
                        let (_, list_id) = n.document().get(ROOT, "list").unwrap().unwrap();

                        let values: Vec<_> = n.document().list_range(list_id, ..).collect();
                        values.windows(2).all(|w| w[0].1 != w[1].1)
                    } else {
                        true
                    }
                })
            },
        )]
    }

    fn record_input(
        &self,
    ) -> fn(
        cfg: &Self::Config,
        history: &Self::History,
        message: stateright::actor::Envelope<&GlobalMsg<Self::App>>,
    ) -> Option<Self::History> {
        |_, h, m| {
            if matches!(m.msg, GlobalMsg::ClientToServer(ApplicationMsg::Input(_))) {
                let mut nh = h.clone();
                nh.push(m.msg.clone());
                Some(nh)
            } else {
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {}

#[cfg(test)]
mod tests {
    use amc::{application::server::SyncMethod, model::ModelOpts};

    use expect_test::expect;

    use super::*;

    #[test]
    fn fully_broken() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            batch_synchronisation: false,
            restarts: false,
            in_sync_check: false,
            save_load_check: false,
            historical_check: false,
            error_free_check: false,
        };
        let moves_opts = MovesOpts { initial_size: 1 };

        amc_test::check_bfs(
            model_opts,
            moves_opts,
            expect![[r#"
                Done states=147, unique=89, max_depth=5
                Discovered "no duplicates when in sync" counterexample Path[4]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(Move(0, 0))) }
                - Deliver { src: Id(4), dst: Id(1), msg: ClientToServer(Input(Move(0, 0))) }
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg4uoy0wBagEZ2/yCVWqEhxxbv8U9m/PGQRtWd/xxKYnwuCsD89CoxwgAAAAAAAAAAAEDAAABCAAAAAAAAAPnCwECAgIRBBMDNAJCA1YDVwFwA3ECcwICAQIBfwEAAX4CfgEBfgMBfgAWYX4BAH8BfwI"] }) }
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg+/vwrgBagEZ2/yCVWqEhxxbv8U9m/PGQRtWd/xxKYnwuCsD89CoxwgAAAAAAAAAAQEDAAABCAAAAAAAAAPnCwECAgIRBBMDNAJCA1YDVwFwA3ECcwICAQIBfwEAAX4CfgEBfgMBfgAWYX4BAH8BfwI"] }) }
                To explore this path try re-running with `explore 4596863851397185902/2059890198849569732/13637464084275399675/4742061144080449335/13795561724444561862`"#]],
        );
    }

    #[test]
    fn fully_broken_sync_messages() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Messages,
            batch_synchronisation: false,
            restarts: false,
            in_sync_check: false,
            save_load_check: false,
            historical_check: false,
            error_free_check: false,
        };
        let moves_opts = MovesOpts { initial_size: 1 };

        amc_test::check_bfs(
            model_opts,
            moves_opts,
            expect![[r#"
                Done states=1211, unique=649, max_depth=9
                Discovered "no duplicates when in sync" counterexample Path[8]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(Move(0, 0))) }
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncMessageRaw { message_bytes: "QgGLqMtMsZ0HkCT1v4Lnu5/A93T+avmorCMyhnXoLhztowABAAYCCgcCvF8A" }) }
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncMessageRaw { message_bytes: "QgEZ2/yCVWqEhxxbv8U9m/PGQRtWd/xxKYnwuCsD89CoxwGLqMtMsZ0HkCT1v4Lnu5/A93T+avmorCMyhnXoLhztowEABQEKByDeAA" }) }
                - Deliver { src: Id(4), dst: Id(1), msg: ClientToServer(Input(Move(0, 0))) }
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncMessageRaw { message_bytes: "QgGLqMtMsZ0HkCT1v4Lnu5/A93T+avmorCMyhnXoLhztowABARnb/IJVaoSHHFu/xT2b88ZBG1Z3/HEpifC4KwPz0KjHBQEKB4I9AXSFb0qDi6jLTAFqARnb/IJVaoSHHFu/xT2b88ZBG1Z3/HEpifC4KwPz0KjHCAAAAAAAAAAAAQMAAAEIAAAAAAAAA+cLAQICAhEEEwM0AkIDVgNXAXADcQJzAgIBAgF/AQABfgJ+AQF+AwF+ABZhfgEAfwF/Ag" }) }
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncMessageRaw { message_bytes: "QgHv78K4b3Sc/TrrMvTnIAggBLNU8+QVc5pKLTSFxsDD7QGLqMtMsZ0HkCT1v4Lnu5/A93T+avmorCMyhnXoLhztowEABgIKB8CsVgF0hW9Kg+/vwrgBagEZ2/yCVWqEhxxbv8U9m/PGQRtWd/xxKYnwuCsD89CoxwgAAAAAAAAAAQEDAAABCAAAAAAAAAPnCwECAgIRBBMDNAJCA1YDVwFwA3ECcwICAQIBfwEAAX4CfgEBfgMBfgAWYX4BAH8BfwI" }) }
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncMessageRaw { message_bytes: "QgKLqMtMsZ0HkCT1v4Lnu5/A93T+avmorCMyhnXoLhzto+/vwrhvdJz9Ousy9OcgCCAEs1Tz5BVzmkotNIXGwMPtAAEB7+/CuG90nP066zL05yAIIASzVPPkFXOaSi00hcbAw+0FAQoHgj0A" }) }
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncMessageRaw { message_bytes: "QgKLqMtMsZ0HkCT1v4Lnu5/A93T+avmorCMyhnXoLhzto+/vwrhvdJz9Ousy9OcgCCAEs1Tz5BVzmkotNIXGwMPtAAEBi6jLTLGdB5Ak9b+C57ufwPd0/mr5qKwjMoZ16C4c7aMFAQoHgMQA" }) }
                To explore this path try re-running with `explore 4596863851397185902/221331385123046224/15499083154062355532/8728495104524905156/8859108831131884740/12085236052126249328/8782270388884776451/9630948498218840186/6468371964391684142`"#]],
        );
    }
}
