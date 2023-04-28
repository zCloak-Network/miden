use super::{fmt, Digest};

// PROXY BLOCK
// ================================================================================================
/// Block for a unknown function call.
///
/// Proxy blocks are used to verify the integrity of a program's hash while keeping parts
/// of the program secret. Fails if executed.
///
/// Hash of a proxy block is not computed but is rather defined at instantiation time.
#[derive(Clone, Debug)]
pub struct Proxy {
    hash: Digest,
}

impl Proxy {
    /// Returns a new [Proxy] block instantiated with the specified code hash.
    pub fn new(code_hash: Digest) -> Self {
        Self { hash: code_hash }
    }

    /// Returns a hash of this code block.
    pub fn hash(&self) -> Digest {
        self.hash
    }
}

impl fmt::Display for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "proxy.{:?}", self.hash) // TODO: use hex, change formatting
    }
}
