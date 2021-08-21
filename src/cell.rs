use crate::adjacency::{are_line_fragments_connecting, Adjacency};
use crate::line_fragment::LineFragment;
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
        match (self, other) {
            // If either Cell is empty, then no connection can be made
            (Cell::Empty, _) | (_, Cell::Empty) => false,
            (Cell::Filled(lf_a), Cell::Filled(lf_b)) => {
                are_line_fragments_connecting(lf_a, adjacency, lf_b)
            }
        }
    }
}
