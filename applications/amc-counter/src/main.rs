//! An example of implementing an application to be tested with AMC.
//!
//! Run with `cargo run --release --bin amc-counter -- --help`
//!
//! The counter that this models is very simple, having an increment and decrement action.

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
use automerge::ReadDoc;
use automerge::ScalarValue;
use automerge::ROOT;
use stateright::actor::ActorModel;
use stateright::Property;
use std::borrow::Cow;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct Counter {
    initial_value: usize,
    counter_type: bool,
    initial_change: bool,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct CounterState {
    value: usize,
    doc: Document,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
enum CounterMsg {
    Increment,
    Decrement,
}

impl Application for Counter {
    type Input = CounterMsg;
    type Output = ();
    type State = CounterState;

    fn init(&self, id: usize) -> Self::State {
        let mut doc = Document::new(id);
        if self.initial_change {
            doc.with_initial_change(|txn| {
                txn.put(ROOT, "counter", ScalarValue::counter(0)).unwrap();
            })
        }
        CounterState {
            value: self.initial_value,
            doc,
        }
    }

    fn execute(&self, state: &mut Cow<Self::State>, input: Self::Input) -> Self::Output {
        match input {
            CounterMsg::Increment => {
                if self.counter_type {
                    let state = state.to_mut();
                    let mut txn = state.doc.transaction();
                    if txn.get(ROOT, "counter").unwrap().is_none() {
                        txn.put(ROOT, "counter", ScalarValue::counter(0)).unwrap();
                    }
                    txn.increment(ROOT, "counter", 1).unwrap();
                    txn.commit();
                } else {
                    let value = state
                        .doc
                        .get(ROOT, "counter")
                        .unwrap()
                        .and_then(|(v, _)| v.to_i64())
                        .unwrap_or_default();
                    let state = state.to_mut();
                    let mut txn = state.doc.transaction();
                    txn.put(ROOT, "counter", value + 1).unwrap();
                    txn.commit();
                }
            }
            CounterMsg::Decrement => {
                if self.counter_type {
                    let state = state.to_mut();
                    let mut txn = state.doc.transaction();
                    if txn.get(ROOT, "counter").unwrap().is_none() {
                        txn.put(ROOT, "counter", ScalarValue::counter(0)).unwrap();
                    }
                    txn.increment(ROOT, "counter", -1).unwrap();
                    txn.commit();
                } else {
                    let value = state
                        .doc
                        .get(ROOT, "counter")
                        .unwrap()
                        .and_then(|(v, _)| v.to_i64())
                        .unwrap_or_default();
                    let state = state.to_mut();
                    let mut txn = state.doc.transaction();
                    txn.put(ROOT, "counter", value - 1).unwrap();
                    txn.commit();
                }
            }
        }
    }
}

impl DerefDocument for CounterState {
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
    /// Number of times to send an increment.
    Inc(u8),
    /// Number of times to send a decrement.
    Dec(u8),
}

impl Drive<Counter> for Driver {
    type State = ();

    fn init(
        &self,
        _application_id: usize,
    ) -> (
        <Self as Drive<Counter>>::State,
        Vec<<Counter as Application>::Input>,
    ) {
        match self.func {
            DriverFunc::Inc(n) => {
                let msgs = (0..n).map(|_| CounterMsg::Increment).collect();
                ((), msgs)
            }
            DriverFunc::Dec(n) => {
                let msgs = (0..n).map(|_| CounterMsg::Decrement).collect();
                ((), msgs)
            }
        }
    }

    fn handle_output(
        &self,
        _state: &mut Cow<Self::State>,
        _output: <Counter as Application>::Output,
    ) -> Vec<<Counter as Application>::Input> {
        Vec::new()
    }
}

#[derive(clap::Args, Debug)]
struct CounterOpts {
    /// Whether to use the built-in counter type, part 1 of a fix.
    #[clap(long, global = true)]
    counter_type: bool,

    /// Whether to initialise the document the same for each application, part 2 of a fix.
    #[clap(long, global = true)]
    initial_change: bool,

    #[clap(long, global = true, default_value = "2")]
    increments: u8,

    #[clap(long, global = true, default_value = "2")]
    decrements: u8,
}

#[derive(clap::Parser, Debug)]
struct Args {
    #[clap(flatten)]
    counter_opts: CounterOpts,

    #[clap(flatten)]
    amc_args: amc::cli::RunArgs,
}

impl ModelBuilder for CounterOpts {
    type App = Counter;

    type Driver = Driver;

    type Config = Config;

    type History = Vec<GlobalMsg<Counter>>;

    fn application(&self, _application: usize) -> Self::App {
        Counter {
            initial_value: 1,
            counter_type: self.counter_type,
            initial_change: self.initial_change,
        }
    }

    fn drivers(&self, _application: usize) -> Vec<Self::Driver> {
        vec![
            Driver {
                func: DriverFunc::Inc(self.increments),
            },
            Driver {
                func: DriverFunc::Dec(self.decrements),
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
        type Prop =
            Property<ActorModel<GlobalActor<Counter, Driver>, Config, Vec<GlobalMsg<Counter>>>>;
        vec![Prop::always("correct value", |_model, state| {
            // When states are in sync, they should have the value of the counter matching that of
            // the combination of increments and decrements.
            if !syncing_done(state) {
                return true;
            }

            let mut expected_value = 0;
            for msg in &state.history {
                match msg.input() {
                    Some(CounterMsg::Increment) => {
                        expected_value += 1;
                    }
                    Some(CounterMsg::Decrement) => {
                        expected_value -= 1;
                    }
                    None => {}
                }
            }

            if let GlobalActorState::Server(s) = &**state.actor_states.first().unwrap() {
                let actual_value = s
                    .document()
                    .get(ROOT, "counter")
                    .unwrap()
                    .and_then(|(v, _)| v.to_i64())
                    .unwrap_or_default();
                return actual_value == expected_value;
            }
            panic!("Couldn't find a server!");
        })]
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
        counter_opts,
        amc_args,
    } = Args::parse();
    amc_args.run(counter_opts);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use amc::{application::server::SyncMethod, model::ModelOpts};
    use stateright::{Checker, Model};

    use expect_test::{expect, Expect};

    use super::*;

    fn check(model_opts: ModelOpts, counter_opts: CounterOpts, expected: Expect) {
        let model = model_opts.to_model(&counter_opts);
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
        let counter_opts = CounterOpts {
            counter_type: false,
            initial_change: false,
            increments: 1,
            decrements: 1,
        };

        check(
            model_opts,
            counter_opts,
            expect![[r#"
            {
                "correct value": [
                    Deliver {
                        src: Id(2),
                        dst: Id(0),
                        msg: ClientToServer(
                            Input(
                                Increment,
                            ),
                        ),
                    },
                    Deliver {
                        src: Id(4),
                        dst: Id(1),
                        msg: ClientToServer(
                            Input(
                                Increment,
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
                                    "hW9Kg9HytnIBLQAIAAAAAAAAAAABAQAAAAYVCTQBQgJWAlcBcAJ/B2NvdW50ZXIBfwF/FAF/AA",
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
                                    "hW9Kg37mM9cBLQAIAAAAAAAAAAEBAQAAAAYVCTQBQgJWAlcBcAJ/B2NvdW50ZXIBfwF/FAF/AA",
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
    fn counter_type_partial_fix() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let counter_opts = CounterOpts {
            counter_type: true,
            initial_change: false,
            increments: 1,
            decrements: 1,
        };

        check(
            model_opts,
            counter_opts,
            expect![[r#"
                {
                    "correct value": [
                        Deliver {
                            src: Id(2),
                            dst: Id(0),
                            msg: ClientToServer(
                                Input(
                                    Increment,
                                ),
                            ),
                        },
                        Deliver {
                            src: Id(4),
                            dst: Id(1),
                            msg: ClientToServer(
                                Input(
                                    Increment,
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
                                        "hW9Kg8uC6w0BOQAIAAAAAAAAAAABAQAAAAgVCTQBQgNWA1cCcANxAnMCAgdjb3VudGVyAn4BBX4YFAABfgABfwB/AQ",
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
                                        "hW9Kg5SFxa4BOQAIAAAAAAAAAAEBAQAAAAgVCTQBQgNWA1cCcANxAnMCAgdjb3VudGVyAn4BBX4YFAABfgABfwB/AQ",
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
    fn intial_change_partial_fix() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let counter_opts = CounterOpts {
            counter_type: false,
            initial_change: true,
            increments: 1,
            decrements: 1,
        };

        check(
            model_opts,
            counter_opts,
            expect![[r#"
                {
                    "correct value": [
                        Deliver {
                            src: Id(2),
                            dst: Id(0),
                            msg: ClientToServer(
                                Input(
                                    Increment,
                                ),
                            ),
                        },
                        Deliver {
                            src: Id(4),
                            dst: Id(1),
                            msg: ClientToServer(
                                Input(
                                    Increment,
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
                                        "hW9Kg/YbISQBXgFSivc63RbdozTgVxdZsTebmtG2LZfGjrMebHARiIr6ywgAAAAAAAAAAAECAAABCAAAAAAAAAPnCBUJNAFCAlYCVwFwAnECcwJ/B2NvdW50ZXIBfwF/FAF/AX8BfwE",
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
                                        "hW9Kg7tkl20BXgFSivc63RbdozTgVxdZsTebmtG2LZfGjrMebHARiIr6ywgAAAAAAAAAAQECAAABCAAAAAAAAAPnCBUJNAFCAlYCVwFwAnECcwJ/B2NvdW50ZXIBfwF/FAF/AX8BfwE",
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
    fn both_fixes() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let counter_opts = CounterOpts {
            counter_type: true,
            initial_change: true,
            increments: 1,
            decrements: 1,
        };

        check(
            model_opts,
            counter_opts,
            expect![[r#"
                {}
            "#]],
        );
    }
}
