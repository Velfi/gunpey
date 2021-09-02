use std::fmt::Display;

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
            Cell::Empty => false,
            Cell::Filled(LineFragment { is_active, .. }) => *is_active,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Cell::Empty => '.',
            Cell::Filled(lf) => lf.to_char(),
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Cell::Empty => ".",
            Cell::Filled(lf) => lf.to_str(),
        }
    }

    pub fn from_char(c: &char) -> Self {
        if *c == '.' {
            Cell::Empty
        } else {
            let lf = LineFragment::from_char(c);
            Cell::Filled(lf)
        }
    }

    pub fn from_str(cell_str: &str) -> Self {
        if cell_str == "." {
            Cell::Empty
        } else {
            let lf = LineFragment::from_str(cell_str);
            Cell::Filled(lf)
        }
    }

    pub fn activate(&mut self) {
        if let Cell::Filled(lf) = self {
            lf.is_active = true;
        }
    }

    pub fn deactivate(&mut self) {
        if let Cell::Filled(lf) = self {
            lf.is_active = false;
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

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &Cell::Empty => ".",
                &Cell::Filled(lf) => lf.to_str(),
            }
        )
    }
}
