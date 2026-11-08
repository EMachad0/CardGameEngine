/// A card definition's place in the binder that parsed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DefId(usize);

impl DefId {
    pub(super) const fn new(idx: usize) -> Self {
        Self(idx)
    }

    pub(super) fn idx(self) -> usize {
        self.0
    }
}
