use stateright::actor::{Actor, Id};

use crate::{
    client::{Client, ClientMsg},
    register::GlobalMsg,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trigger {
    pub func: TriggerState,
    pub server: Id,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TriggerState {
    MapSinglePut { request_count: usize, key: String },
    MapSingleDelete { request_count: usize, key: String },
    ListStartPut { request_count: usize, index: usize },
    ListDelete { request_count: usize, index: usize },
    ListInsert { request_count: usize, index: usize },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TriggerMsg {
    MapSinglePut { key: String },
    MapSingleDelete { key: String },
    ListPut { index: usize },
    ListDelete { index: usize },
    ListInsert { index: usize },
}

impl Actor for Trigger {
    type Msg = GlobalMsg<Client>;

    type State = ();

    fn on_start(
        &self,
        _id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        match &self.func {
            TriggerState::MapSinglePut { request_count, key } => {
                for _ in 0..*request_count {
                    o.send(
                        self.server,
                        GlobalMsg::External(ClientMsg::Request(TriggerMsg::MapSinglePut {
                            key: key.clone(),
                        })),
                    );
                }
            }
            TriggerState::MapSingleDelete { request_count, key } => {
                for _ in 0..*request_count {
                    o.send(
                        self.server,
                        GlobalMsg::External(ClientMsg::Request(TriggerMsg::MapSingleDelete {
                            key: key.clone(),
                        })),
                    );
                }
            }
            TriggerState::ListStartPut {
                request_count,
                index,
            } => {
                for _ in 0..*request_count {
                    o.send(
                        self.server,
                        GlobalMsg::External(ClientMsg::Request(TriggerMsg::ListPut {
                            index: *index,
                        })),
                    );
                }
            }
            TriggerState::ListDelete {
                request_count,
                index,
            } => {
                for _ in 0..*request_count {
                    o.send(
                        self.server,
                        GlobalMsg::External(ClientMsg::Request(TriggerMsg::ListDelete {
                            index: *index,
                        })),
                    );
                }
            }
            TriggerState::ListInsert {
                request_count,
                index,
            } => {
                for _ in 0..*request_count {
                    o.send(
                        self.server,
                        GlobalMsg::External(ClientMsg::Request(TriggerMsg::ListInsert {
                            index: *index,
                        })),
                    );
                }
            }
        }
    }
}
