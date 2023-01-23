//! An example of implementing an application to be tested with AMC.
//!
//! Run with `cargo run --release --bin amc-moves -- --help`
//!
//! This models concurrently moving elements in a list.

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
use automerge::ROOT;
use stateright::actor::ActorModel;
use stateright::Property;
use std::borrow::Cow;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct List {}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct ListState {
    doc: Document,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
enum ListInput {
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

            txn.insert(&list_id, 0, 'a').unwrap();
            txn.insert(&list_id, 1, 'b').unwrap();
            txn.insert(&list_id, 2, 'c').unwrap();
        });

        ListState { doc }
    }

    fn execute(&self, state: &mut Cow<Self::State>, input: Self::Input) -> Self::Output {
        match input {
            ListInput::Move(from, to) => {
                let (_, list_id) = state.document().get(ROOT, "list").unwrap().unwrap();
                let mut txn = state.to_mut().document_mut().transaction();
                assert!(from > to);
                let (item, _) = txn.get(&list_id, from).unwrap().unwrap();
                let item = item.into_scalar().unwrap();
                txn.delete(&list_id, from).unwrap();
                txn.insert(&list_id, to, item).unwrap();
                txn.commit();
            }
        }
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
struct Driver {
    func: DriverFunc,
}

/// Action for the application to perform.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
enum DriverFunc {
    Mover,
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
            DriverFunc::Mover => {
                let msgs = vec![ListInput::Move(2, 0)];
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
struct MovesOpts {}

#[derive(clap::Parser, Debug)]
struct Args {
    #[clap(flatten)]
    moves_opts: MovesOpts,

    #[clap(flatten)]
    amc_args: amc::cli::RunArgs,
}

impl ModelBuilder for MovesOpts {
    type App = List;

    type Driver = Driver;

    type Config = Config;

    type History = Vec<GlobalMsg<List>>;

    fn application(&self, _application: usize) -> Self::App {
        List {}
    }

    fn drivers(&self, _application: usize) -> Vec<Self::Driver> {
        vec![Driver {
            func: DriverFunc::Mover,
        }]
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
struct Config {}

fn main() {
    use clap::Parser;
    let Args {
        moves_opts,
        amc_args,
    } = Args::parse();
    amc_args.run(moves_opts);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use amc::{application::server::SyncMethod, model::ModelOpts};
    use stateright::{Checker, Model};

    use expect_test::{expect, Expect};

    use super::*;

    fn check(model_opts: ModelOpts, moves_opts: MovesOpts, expected: Expect) {
        let model = model_opts.to_model(&moves_opts);
        let checker = model.checker().spawn_bfs().join();

        let discoveries: BTreeMap<_, _> = checker
            .discoveries()
            .into_iter()
            .map(|(n, p)| (n, p.into_actions()))
            .collect();

        expected.assert_debug_eq(&discoveries);
    }

    #[test]
    fn fully_broken() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let moves_opts = MovesOpts {};

        check(
            model_opts,
            moves_opts,
            expect![[r#"
                {
                    "no duplicates when in sync": [
                        Deliver {
                            src: Id(2),
                            dst: Id(0),
                            msg: ClientToServer(
                                Input(
                                    Move(
                                        2,
                                        0,
                                    ),
                                ),
                            ),
                        },
                        Deliver {
                            src: Id(3),
                            dst: Id(1),
                            msg: ClientToServer(
                                Input(
                                    Move(
                                        2,
                                        0,
                                    ),
                                ),
                            ),
                        },
                        Timeout(
                            Id(0),
                        ),
                        Deliver {
                            src: Id(0),
                            dst: Id(1),
                            msg: ServerToServer(
                                SyncChangeRaw {
                                    missing_changes_bytes: [
                                        "hW9Kg5e83wgBagEAOxE/7D9ZQVWV2J2nS/Pc9U9vhtuGIgHrW3eX74NIYwgAAAAAAAAAAAEFAAABCAAAAAAAAAPnCwECAgIRBBMDNAJCA1YDVwFwA3ECcwICAQIBfwEAAX4EfAEBfgMBfgAWY34BAH8BfwQ",
                                    ],
                                },
                            ),
                        },
                        Timeout(
                            Id(1),
                        ),
                        Deliver {
                            src: Id(1),
                            dst: Id(0),
                            msg: ServerToServer(
                                SyncChangeRaw {
                                    missing_changes_bytes: [
                                        "hW9Kgxfm+RUBagEAOxE/7D9ZQVWV2J2nS/Pc9U9vhtuGIgHrW3eX74NIYwgAAAAAAAAAAQEFAAABCAAAAAAAAAPnCwECAgIRBBMDNAJCA1YDVwFwA3ECcwICAQIBfwEAAX4EfAEBfgMBfgAWY34BAH8BfwQ",
                                    ],
                                },
                            ),
                        },
                    ],
                }
            "#]],
        );
    }

    #[test]
    fn fully_broken_sync_messages() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Messages,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let moves_opts = MovesOpts {};

        check(
            model_opts,
            moves_opts,
            expect![[r#"
                {
                    "no duplicates when in sync": [
                        Deliver {
                            src: Id(2),
                            dst: Id(0),
                            msg: ClientToServer(
                                Input(
                                    Move(
                                        2,
                                        0,
                                    ),
                                ),
                            ),
                        },
                        Deliver {
                            src: Id(3),
                            dst: Id(1),
                            msg: ClientToServer(
                                Input(
                                    Move(
                                        2,
                                        0,
                                    ),
                                ),
                            ),
                        },
                        Timeout(
                            Id(0),
                        ),
                        Deliver {
                            src: Id(0),
                            dst: Id(1),
                            msg: ServerToServer(
                                SyncMessageRaw {
                                    message_bytes: "QgGXvN8IVm6aCN89w7vkLMzRLDFLFQThTSRGH7wFSTMXtgABAAYCCgd2+EkA",
                                },
                            ),
                        },
                        Deliver {
                            src: Id(1),
                            dst: Id(0),
                            msg: ServerToServer(
                                SyncMessageRaw {
                                    message_bytes: "QgEX5vkVRcCHHuMqJOYH6QIZlzwkY/kI56cnY9Sfcev5bQGXvN8IVm6aCN89w7vkLMzRLDFLFQThTSRGH7wFSTMXtgEABgIKB7BkWQF0hW9Kgxfm+RUBagEAOxE/7D9ZQVWV2J2nS/Pc9U9vhtuGIgHrW3eX74NIYwgAAAAAAAAAAQEFAAABCAAAAAAAAAPnCwECAgIRBBMDNAJCA1YDVwFwA3ECcwICAQIBfwEAAX4EfAEBfgMBfgAWY34BAH8BfwQ",
                                },
                            ),
                        },
                        Deliver {
                            src: Id(0),
                            dst: Id(1),
                            msg: ServerToServer(
                                SyncMessageRaw {
                                    message_bytes: "QgIX5vkVRcCHHuMqJOYH6QIZlzwkY/kI56cnY9Sfcev5bZe83whWbpoI3z3Du+QszNEsMUsVBOFNJEYfvAVJMxe2AAEBF+b5FUXAhx7jKiTmB+kCGZc8JGP5COenJ2PUn3Hr+W0FAQoHxDoBdIVvSoOXvN8IAWoBADsRP+w/WUFVldidp0vz3PVPb4bbhiIB61t3l++DSGMIAAAAAAAAAAABBQAAAQgAAAAAAAAD5wsBAgICEQQTAzQCQgNWA1cBcANxAnMCAgECAX8BAAF+BHwBAX4DAX4AFmN+AQB/AX8E",
                                },
                            ),
                        },
                        Deliver {
                            src: Id(1),
                            dst: Id(0),
                            msg: ServerToServer(
                                SyncMessageRaw {
                                    message_bytes: "QgIX5vkVRcCHHuMqJOYH6QIZlzwkY/kI56cnY9Sfcev5bZe83whWbpoI3z3Du+QszNEsMUsVBOFNJEYfvAVJMxe2AAECF+b5FUXAhx7jKiTmB+kCGZc8JGP5COenJ2PUn3Hr+W2XvN8IVm6aCN89w7vkLMzRLDFLFQThTSRGH7wFSTMXtgAA",
                                },
                            ),
                        },
                    ],
                }
            "#]],
        );
    }
}
