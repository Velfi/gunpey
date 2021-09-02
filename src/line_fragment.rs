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

    pub fn from_str(lf_str: &str) -> Self {
        let (is_active, kind) = match lf_str {
            "C" => (true, LineFragmentKind::Caret),
            "c" => (false, LineFragmentKind::Caret),
            "I" => (true, LineFragmentKind::InvertedCaret),
            "i" => (false, LineFragmentKind::InvertedCaret),
            "L" => (true, LineFragmentKind::LeftSlash),
            "l" => (false, LineFragmentKind::LeftSlash),
            "R" => (true, LineFragmentKind::RightSlash),
            "r" => (false, LineFragmentKind::RightSlash),
            _ => unreachable!(r#"invalid lf_str "{}""#, lf_str),
        };

        Self { is_active, kind }
    }

    pub fn to_char(&self) -> char {
        self.kind.to_char()
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            &LineFragment { is_active, kind } => match kind {
                LineFragmentKind::Caret if is_active => "C",
                LineFragmentKind::Caret => "c",
                LineFragmentKind::InvertedCaret if is_active => "I",
                LineFragmentKind::InvertedCaret => "i",
                LineFragmentKind::LeftSlash if is_active => "L",
                LineFragmentKind::LeftSlash => "l",
                LineFragmentKind::RightSlash if is_active => "R",
                LineFragmentKind::RightSlash => "r",
            },
        }
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
