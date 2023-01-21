use std::fmt::Debug;

/// A convenience wrapper around some bytes to show them in base64 form when debugging.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Bytes(pub Vec<u8>);

impl Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(&self.0);
        b64.fmt(f)
    }
}
