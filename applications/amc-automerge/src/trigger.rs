use amc::driver::{ApplicationMsg, Drive};
use stateright::actor::{Actor, Id};

use crate::client::App;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Driver {
    pub func: DriverState,
    pub server: Id,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DriverState {
    MapSinglePut { request_count: usize, key: String },
    MapSingleDelete { request_count: usize, key: String },
    ListStartPut { request_count: usize, index: usize },
    ListDelete { request_count: usize, index: usize },
    ListInsert { request_count: usize, index: usize },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DriverMsg {
    MapSinglePut { key: String },
    MapSingleDelete { key: String },
    ListPut { index: usize },
    ListDelete { index: usize },
    ListInsert { index: usize },
}

impl Drive<App> for Driver {}

impl Actor for Driver {
    type Msg = ApplicationMsg<App>;

    type State = ();

    fn on_start(
        &self,
        _id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        match &self.func {
            DriverState::MapSinglePut { request_count, key } => {
                for _ in 0..*request_count {
                    o.send(
                        self.server,
                        ApplicationMsg::Input(DriverMsg::MapSinglePut { key: key.clone() }),
                    );
                }
            }
            DriverState::MapSingleDelete { request_count, key } => {
                for _ in 0..*request_count {
                    o.send(
                        self.server,
                        ApplicationMsg::Input(DriverMsg::MapSingleDelete { key: key.clone() }),
                    );
                }
            }
            DriverState::ListStartPut {
                request_count,
                index,
            } => {
                for _ in 0..*request_count {
                    o.send(
                        self.server,
                        ApplicationMsg::Input(DriverMsg::ListPut { index: *index }),
                    );
                }
            }
            DriverState::ListDelete {
                request_count,
                index,
            } => {
                for _ in 0..*request_count {
                    o.send(
                        self.server,
                        ApplicationMsg::Input(DriverMsg::ListDelete { index: *index }),
                    );
                }
            }
            DriverState::ListInsert {
                request_count,
                index,
            } => {
                for _ in 0..*request_count {
                    o.send(
                        self.server,
                        ApplicationMsg::Input(DriverMsg::ListInsert { index: *index }),
                    );
                }
            }
        }
    }
}
