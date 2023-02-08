#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ScalarValue {
    Bytes(Vec<u8>),
    Str(String),
    Int(i64),
    Uint(u64),
    /// Counter with initial value.
    Counter(i64),
    Timestamp(i64),
    Boolean(bool),
    Null,
}

impl From<automerge::ScalarValue> for ScalarValue {
    fn from(s: automerge::ScalarValue) -> Self {
        match s {
            automerge::ScalarValue::Bytes(b) => Self::Bytes(b),
            automerge::ScalarValue::Str(s) => Self::Str(s.into()),
            automerge::ScalarValue::Int(i) => Self::Int(i),
            automerge::ScalarValue::Uint(u) => Self::Uint(u),
            automerge::ScalarValue::F64(_) => todo!(),
            // TODO: expose counter type in automerge to get initial value
            automerge::ScalarValue::Counter(_) => Self::Counter(0),
            automerge::ScalarValue::Timestamp(t) => Self::Timestamp(t),
            automerge::ScalarValue::Boolean(b) => Self::Boolean(b),
            automerge::ScalarValue::Unknown {
                type_code: _,
                bytes: _,
            } => todo!(),
            automerge::ScalarValue::Null => Self::Null,
        }
    }
}

impl From<ScalarValue> for automerge::ScalarValue {
    fn from(s: ScalarValue) -> Self {
        match s {
            ScalarValue::Bytes(b) => automerge::ScalarValue::Bytes(b),
            ScalarValue::Str(s) => automerge::ScalarValue::Str(s.into()),
            ScalarValue::Int(i) => automerge::ScalarValue::Int(i),
            ScalarValue::Uint(u) => automerge::ScalarValue::Uint(u),
            ScalarValue::Counter(i) => automerge::ScalarValue::counter(i),
            ScalarValue::Timestamp(t) => automerge::ScalarValue::Timestamp(t),
            ScalarValue::Boolean(b) => automerge::ScalarValue::Boolean(b),
            ScalarValue::Null => automerge::ScalarValue::Null,
        }
    }
}

impl ScalarValue {
    pub fn is_counter(&self) -> bool {
        matches!(self, Self::Counter(_))
    }
}
