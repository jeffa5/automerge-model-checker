use amc::driver::Drive;

use crate::client::App;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Driver {
    pub func: DriverState,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DriverState {
    MapSinglePut {
        request_count: usize,
        key: String,
        value: String,
    },
    MapSingleDelete {
        request_count: usize,
        key: String,
    },
    ListStartPut {
        request_count: usize,
        index: usize,
        value: String,
    },
    ListInsert {
        request_count: usize,
        index: usize,
        value: String,
    },
    ListDelete {
        request_count: usize,
        index: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DriverMsg {
    MapSinglePut { key: String, value: String },
    MapSingleDelete { key: String },
    ListPut { index: usize, value: String },
    ListInsert { index: usize, value: String },
    ListDelete { index: usize },
}

impl Drive<App> for Driver {
    type State = ();

    fn init(
        &self,
        _application_id: usize,
    ) -> (
        <Self as Drive<App>>::State,
        Vec<<App as amc::prelude::Application>::Input>,
    ) {
        match &self.func {
            DriverState::MapSinglePut {
                request_count,
                key,
                value,
            } => {
                let msgs = (0..*request_count)
                    .map(|_| DriverMsg::MapSinglePut {
                        key: key.clone(),
                        value: value.clone(),
                    })
                    .collect();
                ((), msgs)
            }
            DriverState::MapSingleDelete { request_count, key } => {
                let msgs = (0..*request_count)
                    .map(|_| DriverMsg::MapSingleDelete { key: key.clone() })
                    .collect();
                ((), msgs)
            }
            DriverState::ListStartPut {
                request_count,
                index,
                value,
            } => {
                let msgs = (0..*request_count)
                    .map(|_| DriverMsg::ListPut {
                        index: *index,
                        value: value.clone(),
                    })
                    .collect();
                ((), msgs)
            }
            DriverState::ListDelete {
                request_count,
                index,
            } => {
                let msgs = (0..*request_count)
                    .map(|_| DriverMsg::ListDelete { index: *index })
                    .collect();
                ((), msgs)
            }
            DriverState::ListInsert {
                request_count,
                index,
                value,
            } => {
                let msgs = (0..*request_count)
                    .map(|_| DriverMsg::ListInsert {
                        index: *index,
                        value: value.clone(),
                    })
                    .collect();
                ((), msgs)
            }
        }
    }

    fn handle_output(
        &self,
        _state: &mut std::borrow::Cow<Self::State>,
        _output: <App as amc::prelude::Application>::Output,
    ) -> Vec<<App as amc::prelude::Application>::Input> {
        vec![]
    }
}
