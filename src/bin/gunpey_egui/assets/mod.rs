use image::io::Reader as ImageReader;
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum Asset {
    ActiveCaret,
    ActiveInvertedCaret,
    ActiveLeftSlash,
    ActiveRightSlash,
    Caret,
    Cursor,
    EmptyCell,
    InvertedCaret,
    LeftSlash,
    RightSlash,
}

pub type RawSprite = (usize, usize, Vec<u8>);

/// A list of assets loaded into memory.
#[derive(Debug)]
pub struct Assets {
    sprites: HashMap<Asset, RawSprite>,
}

impl Assets {
    pub fn sprites(&self) -> &HashMap<Asset, RawSprite> {
        &self.sprites
    }
}

/// Load all static assets into an `Assets` structure
#[rustfmt::skip]
pub fn load_assets() -> Assets {
    use Asset::*;

    let mut sprites = HashMap::new();

    sprites.insert( ActiveCaret, load_png(include_bytes!("active_caret.png")));
    sprites.insert( ActiveInvertedCaret, load_png(include_bytes!("active_inverted_caret.png")));
    sprites.insert( ActiveLeftSlash, load_png(include_bytes!("active_left_slash.png")));
    sprites.insert( ActiveRightSlash, load_png(include_bytes!("active_right_slash.png")));
    sprites.insert( Caret, load_png(include_bytes!("caret.png")));
    sprites.insert( Cursor, load_png(include_bytes!("cursor.png")));
    sprites.insert( EmptyCell, load_png(include_bytes!("empty_cell.png")));
    sprites.insert( InvertedCaret, load_png(include_bytes!("inverted_caret.png")));
    sprites.insert( LeftSlash, load_png(include_bytes!("left_slash.png")));
    sprites.insert( RightSlash, load_png(include_bytes!("right_slash.png")));

    Assets { sprites }
}

/// Convert PNG data to raw pixels
fn load_png(png: &[u8]) -> RawSprite {
    let img = ImageReader::new(Cursor::new(png))
        .with_guessed_format()
        .expect("failed to guess file format of sprite")
        .decode()
        .expect("failed to decode sprite")
        .to_rgba8();

    (img.width() as usize, img.height() as usize, img.into_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png() {
        let (width, height, pixels) = load_png(include_bytes!("test_image.png"));
        let expected = vec![
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255,
        ];

        assert_eq!(width, 5, "Width differs");
        assert_eq!(height, 5, "Height differs");
        assert_eq!(pixels, expected, "Pixels differ");
    }
}
