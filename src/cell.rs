use crate::grid_pos::Adjacency;
use crate::line_fragment::{LineFragment, LineFragmentKind};
use druid::Data;

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum Cell {
    Filled(LineFragment),
    Empty,
}

impl Cell {
    pub fn is_empty(&self) -> bool {
        self == &Cell::Empty
    }

    pub fn is_active(&self) -> bool {
        match self {
            Cell::Filled(LineFragment { is_active, .. }) => *is_active,
            Cell::Empty => false,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Cell::Filled(lf) => lf.to_char(),
            Cell::Empty => '.',
        }
    }

    pub fn from_char(c: &char) -> Self {
        match c {
            '.' => Cell::Empty,
            c => {
                let lf = LineFragment::from_char(c);
                Cell::Filled(lf)
            }
        }
    }

    pub fn activate(&mut self) {
        match self {
            Cell::Filled(lf) => {
                lf.is_active = true;
            }
            _ => (),
        }
    }

    pub fn deactivate(&mut self) {
        match self {
            Cell::Filled(lf) => {
                lf.is_active = false;
            }
            _ => (),
        }
    }

    pub fn is_connected_to(&self, other: &Cell, adjacency: Adjacency) -> bool {
        use Adjacency::*;
        use LineFragmentKind::*;

        // TODO I feel bad about this, please help me
        // Some of this will be much nicer once the experimental or-patterns syntax is stabilized
        // https://github.com/rust-lang/rust/issues/54883
        match (self, other) {
            // If either Cell is empty, then no connection can be made
            (Cell::Empty, _) | (_, Cell::Empty) => false,
            (Cell::Filled(lf_a), Cell::Filled(lf_b)) => match (lf_a.kind, lf_b.kind, adjacency) {
                (_, _, Same) => unreachable!("please don't ask if a cell is connected to itself"),
                (_, _, NotAdjacent) => false,
                (Caret, _, AboveLeft) | (Caret, _, Above) | (Caret, _, AboveRight) => false,
                (Caret, Caret, _) => false,
                (Caret, InvertedCaret, Below)
                | (Caret, InvertedCaret, BelowLeft)
                | (Caret, InvertedCaret, BelowRight) => true,
                (Caret, InvertedCaret, Left) | (Caret, InvertedCaret, Right) => false,
                (Caret, LeftSlash, Below)
                | (Caret, LeftSlash, BelowRight)
                | (Caret, LeftSlash, Left) => true,
                (Caret, LeftSlash, BelowLeft) | (Caret, LeftSlash, Right) => false,
                (Caret, RightSlash, Below)
                | (Caret, RightSlash, BelowLeft)
                | (Caret, RightSlash, Right) => true,
                (Caret, RightSlash, Left) | (Caret, RightSlash, BelowRight) => false,
                (InvertedCaret, _, BelowLeft)
                | (InvertedCaret, _, Below)
                | (InvertedCaret, _, BelowRight) => false,
                (InvertedCaret, InvertedCaret, _) => false,
                (InvertedCaret, Caret, Above)
                | (InvertedCaret, Caret, AboveLeft)
                | (InvertedCaret, Caret, AboveRight) => true,
                (InvertedCaret, Caret, Left) | (InvertedCaret, Caret, Right) => false,
                (InvertedCaret, LeftSlash, Above)
                | (InvertedCaret, LeftSlash, AboveRight)
                | (InvertedCaret, LeftSlash, Left) => true,
                (InvertedCaret, LeftSlash, AboveLeft) | (InvertedCaret, LeftSlash, Right) => false,
                (InvertedCaret, RightSlash, Above)
                | (InvertedCaret, RightSlash, AboveLeft)
                | (InvertedCaret, RightSlash, Right) => true,
                (InvertedCaret, RightSlash, Left) | (InvertedCaret, RightSlash, AboveRight) => {
                    false
                }
                (LeftSlash, _, BelowLeft) | (LeftSlash, _, AboveRight) => false,
                (LeftSlash, Caret, Above)
                | (LeftSlash, Caret, AboveLeft)
                | (LeftSlash, Caret, Right) => true,
                (LeftSlash, Caret, Below)
                | (LeftSlash, Caret, Left)
                | (LeftSlash, Caret, BelowRight) => false,
                (LeftSlash, InvertedCaret, Below)
                | (LeftSlash, InvertedCaret, BelowRight)
                | (LeftSlash, InvertedCaret, Left) => true,
                (LeftSlash, InvertedCaret, Above)
                | (LeftSlash, InvertedCaret, AboveLeft)
                | (LeftSlash, InvertedCaret, Right) => false,
                (LeftSlash, LeftSlash, AboveLeft) | (LeftSlash, LeftSlash, BelowRight) => true,
                (LeftSlash, LeftSlash, _) => false,
                (LeftSlash, RightSlash, Above)
                | (LeftSlash, RightSlash, Below)
                | (LeftSlash, RightSlash, Left)
                | (LeftSlash, RightSlash, Right) => true,
                (LeftSlash, RightSlash, AboveLeft) | (LeftSlash, RightSlash, BelowRight) => false,
                (RightSlash, _, AboveLeft) | (RightSlash, _, BelowRight) => false,
                (RightSlash, Caret, Above)
                | (RightSlash, Caret, AboveRight)
                | (RightSlash, Caret, Left) => true,
                (RightSlash, Caret, Below)
                | (RightSlash, Caret, BelowLeft)
                | (RightSlash, Caret, Right) => false,
                (RightSlash, InvertedCaret, Below)
                | (RightSlash, InvertedCaret, BelowLeft)
                | (RightSlash, InvertedCaret, Right) => true,
                (RightSlash, InvertedCaret, Above)
                | (RightSlash, InvertedCaret, Left)
                | (RightSlash, InvertedCaret, AboveRight) => false,
                (RightSlash, LeftSlash, Above)
                | (RightSlash, LeftSlash, Below)
                | (RightSlash, LeftSlash, Left)
                | (RightSlash, LeftSlash, Right) => true,
                (RightSlash, LeftSlash, AboveRight) | (RightSlash, LeftSlash, BelowLeft) => false,
                (RightSlash, RightSlash, BelowLeft) | (RightSlash, RightSlash, AboveRight) => true,
                (RightSlash, RightSlash, _) => false,
            },
        }
    }
}
