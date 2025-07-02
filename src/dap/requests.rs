#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct RequestId(u64);

impl RequestId {
    pub fn new(seq: u64) -> Self {
        Self(seq)
    }
}