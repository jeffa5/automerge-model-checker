use amc::driver::Drive;

use crate::{client::App, scalar::ScalarValue};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Driver {
    pub func: DriverState,
    pub repeats: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DriverState {
    MapSinglePut {
        key: String,
        value: ScalarValue,
    },
    MapSingleDelete {
        key: String,
    },
    ListPut {
        index: usize,
        value: ScalarValue,
    },
    ListInsert {
        index: usize,
        value: ScalarValue,
    },
    ListDelete {
        index: usize,
    },
    ListSplice {
        index: usize,
        delete: usize,
        values: Vec<ScalarValue>,
    },
    TextPut {
        index: usize,
        value: String,
    },
    TextInsert {
        index: usize,
        value: String,
    },
    TextDelete {
        index: usize,
    },
    TextSplice {
        index: usize,
        delete: usize,
        text: String,
    },
    /// Increment a counter inside a map.
    MapIncrement {
        key: String,
        by: i64,
    },
    /// Increment a counter inside a list.
    ListIncrement {
        index: usize,
        by: i64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DriverMsg {
    MapSinglePut {
        key: String,
        value: ScalarValue,
    },
    MapSingleDelete {
        key: String,
    },
    MapIncrement {
        key: String,
        by: i64,
    },
    ListPut {
        index: usize,
        value: ScalarValue,
    },
    ListInsert {
        index: usize,
        value: ScalarValue,
    },
    ListDelete {
        index: usize,
    },
    ListSplice {
        index: usize,
        delete: usize,
        values: Vec<ScalarValue>,
    },
    ListIncrement {
        index: usize,
        by: i64,
    },
    TextPut {
        index: usize,
        value: String,
    },
    TextInsert {
        index: usize,
        value: String,
    },
    TextDelete {
        index: usize,
    },
    TextSplice {
        index: usize,
        delete: usize,
        text: String,
    },
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
        let msgs = match &self.func {
            DriverState::MapSinglePut { key, value } => {
                vec![DriverMsg::MapSinglePut {
                    key: key.clone(),
                    value: value.clone(),
                }]
            }
            DriverState::MapSingleDelete { key } => {
                vec![DriverMsg::MapSingleDelete { key: key.clone() }]
            }
            DriverState::ListPut { index, value } => {
                vec![DriverMsg::ListPut {
                    index: *index,
                    value: value.clone(),
                }]
            }
            DriverState::ListDelete { index } => {
                vec![DriverMsg::ListDelete { index: *index }]
            }
            DriverState::ListInsert { index, value } => {
                vec![DriverMsg::ListInsert {
                    index: *index,
                    value: value.clone(),
                }]
            }
            DriverState::ListSplice {
                index,
                delete,
                values,
            } => {
                vec![DriverMsg::ListSplice {
                    index: *index,
                    delete: *delete,
                    values: values.clone(),
                }]
            }
            DriverState::TextPut { index, value } => {
                vec![DriverMsg::TextPut {
                    index: *index,
                    value: value.clone(),
                }]
            }
            DriverState::TextDelete { index } => {
                vec![DriverMsg::TextDelete { index: *index }]
            }
            DriverState::TextInsert { index, value } => {
                vec![DriverMsg::TextInsert {
                    index: *index,
                    value: value.clone(),
                }]
            }
            DriverState::TextSplice {
                index,
                delete,
                text,
            } => {
                vec![DriverMsg::TextSplice {
                    index: *index,
                    delete: *delete,
                    text: text.clone(),
                }]
            }
            DriverState::MapIncrement { key, by } => {
                vec![DriverMsg::MapIncrement {
                    key: key.clone(),
                    by: *by,
                }]
            }
            DriverState::ListIncrement { index, by } => {
                vec![DriverMsg::ListIncrement {
                    index: *index,
                    by: *by,
                }]
            }
        };
        let mut all_msgs = Vec::new();
        for _ in 0..self.repeats {
            all_msgs.append(&mut msgs.clone());
        }
        ((), all_msgs)
    }

    fn handle_output(
        &self,
        _state: &mut std::borrow::Cow<Self::State>,
        _output: <App as amc::prelude::Application>::Output,
    ) -> Vec<<App as amc::prelude::Application>::Input> {
        vec![]
    }
}
