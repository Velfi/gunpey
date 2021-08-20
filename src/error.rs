use crate::grid_pos::GridPos;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GunpeyLibError {
    #[error("can't swap tiles a={0} and b={1} because one of those indexes is out of bounds")]
    CantSwapBadIndex(usize, usize),
    #[error("can't swap tiles a={0} and b={1} because they are the same tile")]
    CantSwapSameIndex(usize, usize),
    #[error("can't swap tiles a={0} and b={1} because one of those indexes is out of bounds")]
    CantSwapBadPosition(GridPos, GridPos),
    #[error("can't swap tiles a={0} and b={1} because they are the same tile")]
    CantSwapSamePositon(GridPos, GridPos),
    #[error(
        "invalid row size, input row length is {0} which does not equal expected row length of {1}"
    )]
    InvalidRowLength(usize, usize),
}
