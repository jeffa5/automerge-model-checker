use amc::driver::Drive;

use crate::{client::App, scalar::ScalarValue};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Driver {
    pub func: DriverState,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DriverState {
    MapSinglePut { key: String, value: ScalarValue },
    MapSingleDelete { key: String },
    ListPut { index: usize, value: ScalarValue },
    ListInsert { index: usize, value: ScalarValue },
    ListDelete { index: usize },
    TextPut { index: usize, value: String },
    TextInsert { index: usize, value: String },
    TextDelete { index: usize },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DriverMsg {
    MapSinglePut { key: String, value: ScalarValue },
    MapSingleDelete { key: String },
    ListPut { index: usize, value: ScalarValue },
    ListInsert { index: usize, value: ScalarValue },
    ListDelete { index: usize },
    TextPut { index: usize, value: String },
    TextInsert { index: usize, value: String },
    TextDelete { index: usize },
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
            DriverState::MapSinglePut { key, value } => {
                let msgs = vec![DriverMsg::MapSinglePut {
                    key: key.clone(),
                    value: value.clone(),
                }];
                ((), msgs)
            }
            DriverState::MapSingleDelete { key } => {
                let msgs = vec![DriverMsg::MapSingleDelete { key: key.clone() }];
                ((), msgs)
            }
            DriverState::ListPut { index, value } => {
                let msgs = vec![DriverMsg::ListPut {
                    index: *index,
                    value: value.clone(),
                }];
                ((), msgs)
            }
            DriverState::ListDelete { index } => {
                let msgs = vec![DriverMsg::ListDelete { index: *index }];
                ((), msgs)
            }
            DriverState::ListInsert { index, value } => {
                let msgs = vec![DriverMsg::ListInsert {
                    index: *index,
                    value: value.clone(),
                }];
                ((), msgs)
            }
            DriverState::TextPut { index, value } => {
                let msgs = vec![DriverMsg::TextPut {
                    index: *index,
                    value: value.clone(),
                }];
                ((), msgs)
            }
            DriverState::TextDelete { index } => {
                let msgs = vec![DriverMsg::TextDelete { index: *index }];
                ((), msgs)
            }
            DriverState::TextInsert { index, value } => {
                let msgs = vec![DriverMsg::TextInsert {
                    index: *index,
                    value: value.clone(),
                }];
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
