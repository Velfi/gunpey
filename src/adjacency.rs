use crate::{
    grid_pos::GridPos,
    line_fragment::{LineFragment, LineFragmentKind},
};
use std::fmt::Display;

pub fn are_line_fragments_connecting(
    lf_a: &LineFragment,
    adjacency: Adjacency,
    lf_b: &LineFragment,
) -> bool {
    use Adjacency::*;
    use LineFragmentKind::*;

    if adjacency == Same {
        return true;
    } else if adjacency == NotAdjacent {
        return false;
    }

    match lf_a.kind {
        Caret => {
            if adjacency == AboveLeft || adjacency == Above || adjacency == AboveRight {
                return false;
            }

            match lf_b.kind {
                Caret => [Left, Right].contains(&adjacency),
                InvertedCaret => [BelowLeft, Below, BelowRight].contains(&adjacency),
                LeftSlash => [Left, Below, BelowRight].contains(&adjacency),
                RightSlash => [Right, Below, BelowLeft].contains(&adjacency),
            }
        }
        InvertedCaret => {
            if adjacency == BelowLeft || adjacency == Below || adjacency == BelowRight {
                return false;
            }

            match lf_b.kind {
                Caret => [AboveLeft, Above, AboveRight].contains(&adjacency),
                InvertedCaret => [Left, Right].contains(&adjacency),
                LeftSlash => [Right, Above, AboveLeft].contains(&adjacency),
                RightSlash => [Left, Above, AboveRight].contains(&adjacency),
            }
        }
        LeftSlash => {
            if adjacency == BelowLeft || adjacency == AboveRight {
                return false;
            }

            match lf_b.kind {
                Caret => [AboveLeft, Above, Right].contains(&adjacency),
                InvertedCaret => [Left, Below, BelowRight].contains(&adjacency),
                LeftSlash => [AboveLeft, BelowRight].contains(&adjacency),
                RightSlash => [Left, Right, Above, Below].contains(&adjacency),
            }
        }
        RightSlash => {
            if adjacency == AboveLeft || adjacency == BelowRight {
                return false;
            }

            match lf_b.kind {
                Caret => [AboveRight, Above, Left].contains(&adjacency),
                InvertedCaret => [Right, Below, BelowLeft].contains(&adjacency),
                LeftSlash => [Left, Right, Above, Below].contains(&adjacency),
                RightSlash => [AboveRight, BelowLeft].contains(&adjacency),
            }
        }
    }
}

pub fn adjacency_of_grid_positions(gp_a: GridPos, gp_b: GridPos) -> Adjacency {
    match gp_a - gp_b {
        GridPos { x: 0, y: 0 } => Adjacency::Same,
        GridPos { x: 1, y: -1 } => Adjacency::AboveLeft,
        GridPos { x: 0, y: -1 } => Adjacency::Above,
        GridPos { x: -1, y: -1 } => Adjacency::AboveRight,
        GridPos { x: -1, y: 0 } => Adjacency::Right,
        GridPos { x: -1, y: 1 } => Adjacency::BelowRight,
        GridPos { x: 0, y: 1 } => Adjacency::Below,
        GridPos { x: 1, y: 1 } => Adjacency::BelowLeft,
        GridPos { x: 1, y: 0 } => Adjacency::Left,
        _ => Adjacency::NotAdjacent,
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

impl Display for Adjacency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Adjacency::Right => "right",
                Adjacency::BelowLeft => "lower left",
                Adjacency::BelowRight => "lower right",
                Adjacency::Below => "below",
                Adjacency::Left => "left",
                Adjacency::AboveLeft => "upper left",
                Adjacency::AboveRight => "upper right",
                Adjacency::Above => "above",
                Adjacency::Same => "same",
                Adjacency::NotAdjacent => "non-adjacent",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_not_connect_1() {
        let a = LineFragment::from_char(&'/');
        let b = LineFragment::from_char(&'/');
        let adjacency = Adjacency::Right;

        assert_eq!(false, are_line_fragments_connecting(&a, adjacency, &b));
    }

    #[test]
    fn test_should_not_connect_2() {
        let a = LineFragment::from_char(&'/');
        let b = LineFragment::from_char(&'/');
        let adjacency = Adjacency::BelowRight;

        assert_eq!(false, are_line_fragments_connecting(&a, adjacency, &b));
    }

    #[test]
    fn test_should_not_connect_3() {
        let a = LineFragment::from_char(&'/');
        let b = LineFragment::from_char(&'/');
        let adjacency = Adjacency::Below;

        assert_eq!(false, are_line_fragments_connecting(&a, adjacency, &b));
    }

    #[test]
    fn test_should_connect_1() {
        let a = LineFragment::from_char(&'/');
        let b = LineFragment::from_char(&'/');
        let adjacency = Adjacency::BelowLeft;

        assert_eq!(true, are_line_fragments_connecting(&a, adjacency, &b));
    }

    #[test]
    fn test_should_connect_2() {
        let a = LineFragment::from_char(&'∧');
        let b = LineFragment::from_char(&'∨');
        let adjacency = Adjacency::Below;

        assert_eq!(true, are_line_fragments_connecting(&a, adjacency, &b));
    }

    #[test]
    fn test_should_connect_3() {
        let a = LineFragment::from_char(&'∧');
        let b = LineFragment::from_char(&'∧');
        let adjacency = Adjacency::Left;

        assert_eq!(true, are_line_fragments_connecting(&a, adjacency, &b));
    }

    #[test]
    fn test_should_connect_4() {
        let a = LineFragment::from_char(&'∨');
        let b = LineFragment::from_char(&'∨');
        let adjacency = Adjacency::Right;

        assert_eq!(true, are_line_fragments_connecting(&a, adjacency, &b));
    }
}
