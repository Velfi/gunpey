use crate::grid_pos::GridPos;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GunpeyLibError {
    #[error("can't swap tiles by indexs a={a} and b={b} because one of those indexes is out of bounds (length: {length})")]
    CantSwapBadIndex { a: usize, b: usize, length: usize },
    #[error("can't swap tiles a={0} and b={1} because they are the same tile")]
    CantSwapSameIndex(usize, usize),
    #[error("can't swap tiles by positions a={a} and b={b} because one of those positions is out of bounds (width: {width}, height: {height})")]
    CantSwapBadPosition {
        a: GridPos,
        b: GridPos,
        width: usize,
        height: usize,
    },
    #[error("can't swap tiles a={0} and b={1} because they are the same tile")]
    CantSwapSamePositon(GridPos, GridPos),
    #[error(
        "invalid row size, input row length is {0} which does not equal expected row length of {1}"
    )]
    InvalidRowLength(usize, usize),
}
