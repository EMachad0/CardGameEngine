#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetId(pub usize);

/// It has no values, so a `Targeted` selector that holds it can't be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoTarget {}
