use crate::adjacency::Adjacency;
use std::fmt::{Debug, Display};
use std::ops::{Add, Sub};

pub fn gp(x: isize, y: isize) -> GridPos {
    GridPos::new(x, y)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GridPos {
    pub y: isize,
    pub x: isize,
}

impl GridPos {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    pub fn to_tuple(self) -> (isize, isize) {
        let GridPos { y, x } = self;
        (x, y)
    }

    #[rustfmt::skip]
    pub fn adjacency(&self, other: &Self) -> Adjacency {
        match (
            self.x as i64 - other.x as i64,
            self.y as i64 - other.y as i64,
        ) {
            ( 0,  0) => Adjacency::Same,
            ( 0, -1) => Adjacency::Above,
            ( 0,  1) => Adjacency::Below,
            (-1, -1) => Adjacency::AboveRight,
            (-1,  0) => Adjacency::Right,
            (-1,  1) => Adjacency::BelowRight,
            ( 1, -1) => Adjacency::AboveLeft,
            ( 1,  0) => Adjacency::Left,
            ( 1,  1) => Adjacency::BelowLeft,
            ( _,  _) => Adjacency::NotAdjacent,
        }
    }
}

impl Debug for GridPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GridPos {{x: {}, y: {}}}", self.x, self.y)
    }
}

impl Display for GridPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

impl Add for GridPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        GridPos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for GridPos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        GridPos {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_same_adjacency() {
        let pos_a = GridPos::new(0, 0);
        let pos_b = pos_a.clone();
        let actual = pos_a.adjacency(&pos_b);

        assert_eq!(Adjacency::Same, actual);
    }

    #[test]
    fn test_is_above_adjacency() {
        let pos_a = GridPos::new(0, 0);
        let pos_b = GridPos::new(0, 1);
        let actual = pos_a.adjacency(&pos_b);

        assert_eq!(Adjacency::Above, actual);
    }

    #[test]
    fn test_is_below_adjacency() {
        let pos_a = GridPos::new(0, 1);
        let pos_b = GridPos::new(0, 0);
        let actual = pos_a.adjacency(&pos_b);

        assert_eq!(Adjacency::Below, actual);
    }

    #[test]
    fn test_is_above_left_adjacency() {
        let pos_a = GridPos::new(1, 0);
        let pos_b = GridPos::new(0, 1);
        let actual = pos_a.adjacency(&pos_b);

        assert_eq!(Adjacency::AboveLeft, actual);
    }

    #[test]
    fn test_is_below_left_adjacency() {
        let pos_a = GridPos::new(1, 1);
        let pos_b = GridPos::new(0, 0);
        let actual = pos_a.adjacency(&pos_b);

        assert_eq!(Adjacency::BelowLeft, actual);
    }

    #[test]
    fn test_is_left_adjacency() {
        let pos_a = GridPos::new(1, 0);
        let pos_b = GridPos::new(0, 0);
        let actual = pos_a.adjacency(&pos_b);

        assert_eq!(Adjacency::Left, actual);
    }

    #[test]
    fn test_is_above_right_adjacency() {
        let pos_a = GridPos::new(0, 0);
        let pos_b = GridPos::new(1, 1);
        let actual = pos_a.adjacency(&pos_b);

        assert_eq!(Adjacency::AboveRight, actual);
    }

    #[test]
    fn test_is_below_right_adjacency() {
        let pos_a = GridPos::new(0, 1);
        let pos_b = GridPos::new(1, 0);
        let actual = pos_a.adjacency(&pos_b);

        assert_eq!(Adjacency::BelowRight, actual);
    }

    #[test]
    fn test_is_right_adjacency() {
        let pos_a = GridPos::new(0, 0);
        let pos_b = GridPos::new(1, 0);
        let actual = pos_a.adjacency(&pos_b);

        assert_eq!(Adjacency::Right, actual);
    }

    #[test]
    fn test_is_not_adjacent() {
        let pos_a = GridPos::new(0, 2);
        let pos_b = GridPos::new(2, 0);
        let actual = pos_a.adjacency(&pos_b);

        assert_eq!(Adjacency::NotAdjacent, actual);
    }
}
