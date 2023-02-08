use amc::driver::Drive;

use crate::{client::App, scalar::ScalarValue};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Driver {
    pub func: DriverState,
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
            DriverState::ListSplice {
                index,
                delete,
                values,
            } => {
                let msgs = vec![DriverMsg::ListSplice {
                    index: *index,
                    delete: *delete,
                    values: values.clone(),
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
            DriverState::TextSplice {
                index,
                delete,
                text,
            } => {
                let msgs = vec![DriverMsg::TextSplice {
                    index: *index,
                    delete: *delete,
                    text: text.clone(),
                }];
                ((), msgs)
            }
            DriverState::MapIncrement { key, by } => {
                let msgs = vec![DriverMsg::MapIncrement {
                    key: key.clone(),
                    by: *by,
                }];
                ((), msgs)
            }
            DriverState::ListIncrement { index, by } => {
                let msgs = vec![DriverMsg::ListIncrement {
                    index: *index,
                    by: *by,
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
