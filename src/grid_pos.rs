use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GridPos {
    pub row: usize,
    pub column: usize,
}

impl GridPos {
    pub fn new(column: usize, row: usize) -> Self {
        Self { row, column }
    }

    pub fn to_tuple(self) -> (usize, usize) {
        let GridPos { row, column } = self;
        (column, row)
    }

    #[rustfmt::skip]
    pub fn adjacency(&self, other: &Self) -> Adjacency {
        match (
            self.column as i64 - other.column as i64,
            self.row as i64 - other.row as i64,
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

impl Display for GridPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {})", self.column, self.row)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[rustfmt::skip]
pub enum Adjacency {
    AboveLeft, Above, AboveRight,
    Left,      Same,  Right,
    BelowLeft, Below, BelowRight,
    NotAdjacent,
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
