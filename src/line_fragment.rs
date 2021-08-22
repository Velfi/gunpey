use druid::Data;
use rand::{distributions::Standard, prelude::Distribution, Rng};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Data, PartialEq)]
pub enum LineFragmentKind {
    Caret,
    InvertedCaret,
    LeftSlash,
    RightSlash,
}

impl Display for LineFragmentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LineFragmentKind::Caret => "caret",
                LineFragmentKind::InvertedCaret => "inverted caret",
                LineFragmentKind::LeftSlash => "left slash",
                LineFragmentKind::RightSlash => "right slash",
            }
        )
    }
}

impl LineFragmentKind {
    pub fn from_char(c: &char) -> Self {
        match c {
            '∧' => LineFragmentKind::Caret,
            '∨' => LineFragmentKind::InvertedCaret,
            '\\' => LineFragmentKind::LeftSlash,
            '/' => LineFragmentKind::RightSlash,
            _ => unreachable!(),
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            LineFragmentKind::Caret => '∧',
            LineFragmentKind::InvertedCaret => '∨',
            LineFragmentKind::LeftSlash => '\\',
            LineFragmentKind::RightSlash => '/',
        }
    }
}

impl Distribution<LineFragmentKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> LineFragmentKind {
        match rng.gen_range(0..4) {
            0 => LineFragmentKind::Caret,
            1 => LineFragmentKind::InvertedCaret,
            2 => LineFragmentKind::LeftSlash,
            3 => LineFragmentKind::RightSlash,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Data, PartialEq)]
pub struct LineFragment {
    pub kind: LineFragmentKind,
    pub is_active: bool,
}

impl LineFragment {
    pub fn from_char(c: &char) -> Self {
        let kind = LineFragmentKind::from_char(c);

        Self {
            is_active: false,
            kind,
        }
    }

    pub fn to_char(&self) -> char {
        self.kind.to_char()
    }
}

impl Distribution<LineFragment> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> LineFragment {
        LineFragment {
            kind: rng.gen(),
            is_active: false,
        }
    }
}
