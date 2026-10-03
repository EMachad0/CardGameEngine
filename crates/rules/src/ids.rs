//! Identity newtypes.

/// A player's identity, separate from seat order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(usize);

impl PlayerId {
    pub(super) const fn new(idx: usize) -> Self {
        Self(idx)
    }

    pub(super) fn idx(&self) -> usize {
        self.0
    }
}
