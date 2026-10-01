use crate::PlayerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Won(PlayerId),
    Draw,
}
