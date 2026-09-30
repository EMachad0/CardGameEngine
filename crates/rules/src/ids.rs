//! Identity newtypes.

/// A player's identity, separate from seat order: deck `i` belongs to
/// `PlayerId::new(i)` no matter who goes first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(usize);

impl PlayerId {
    /// `const`, so callers can write `const P0: PlayerId = PlayerId::new(0);`.
    pub const fn new(idx: usize) -> Self {
        Self(idx)
    }

    pub fn idx(&self) -> usize {
        self.0
    }
}
