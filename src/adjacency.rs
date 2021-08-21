use crate::line_fragment::{LineFragment, LineFragmentKind};
use bitflags::bitflags;

// based on a playground example I wrote
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=e46ada6ee5eee8964198b758649b51dd

bitflags! {
    struct Flags: u8 {
        /// 0   1
        ///   /   => 0b0000_0110
        /// 1   0

        const ABOVE_LEFT   = 0b0000_1000;
        const ABOVE_RIGHT  = 0b0000_0100;
        const BELOW_LEFT   = 0b0000_0010;
        const BELOW_RIGHT  = 0b0000_0001;
        const SAME         = 0b0000_1111;
        const NOT_ADJACENT = 0b0000_0000;

        const CARET          = Self::BELOW_LEFT.bits  | Self::BELOW_RIGHT.bits;
        const INVERTED_CARET = Self::ABOVE_LEFT.bits  | Self::ABOVE_RIGHT.bits;
        const LEFT_SLASH     = Self::ABOVE_LEFT.bits  | Self::BELOW_RIGHT.bits;
        const RIGHT_SLASH    = Self::ABOVE_RIGHT.bits | Self::BELOW_LEFT.bits;
        const EMPTY          = Self::NOT_ADJACENT.bits;
    }
}

fn flag_from_line_fragment(lf: &LineFragment) -> Flags {
    match lf.kind {
        LineFragmentKind::Caret => Flags::CARET,
        LineFragmentKind::InvertedCaret => Flags::INVERTED_CARET,
        LineFragmentKind::LeftSlash => Flags::LEFT_SLASH,
        LineFragmentKind::RightSlash => Flags::RIGHT_SLASH,
    }
}

pub fn are_line_fragments_connecting(
    lf_a: &LineFragment,
    adjacency: Adjacency,
    lf_b: &LineFragment,
) -> bool {
    if adjacency == Adjacency::Same {
        return true;
    } else if adjacency == Adjacency::NotAdjacent {
        return false;
    }

    let a = flag_from_line_fragment(lf_a);
    let b = flag_from_line_fragment(lf_b);

    let intersection = (a.bits << 4) | b.bits;
    let test_corners = adjacency.into_test_corners_bitmask();

    // TODO is there a bitwise op for this test?
    (test_corners & intersection) == test_corners
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[rustfmt::skip]
pub enum Adjacency {
    AboveLeft, Above, AboveRight,
    Left,      Same,  Right,
    BelowLeft, Below, BelowRight,
    NotAdjacent,
}

impl Adjacency {
    pub fn into_test_corners_bitmask(self) -> u8 {
        match self {
            Adjacency::Right => 0b0101_1010,
            Adjacency::BelowLeft => 0b0010_0100,
            Adjacency::BelowRight => 0b0001_1000,
            Adjacency::Below => 0b0011_1100,
            Adjacency::Left => 0b1010_0101,
            Adjacency::AboveLeft => 0b1000_0001,
            Adjacency::AboveRight => 0b0100_0010,
            Adjacency::Above => 0b1100_0011,
            Adjacency::Same => 0b1111_1111,
            Adjacency::NotAdjacent => 0b0000_0000,
        }
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
}
