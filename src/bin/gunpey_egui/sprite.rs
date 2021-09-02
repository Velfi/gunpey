use egui::Pos2;
use line_drawing::Bresenham;

use crate::assets::{Asset, Assets};

/// Sprites can be drawn and procedurally generated.
///
/// A `Sprite` owns its pixel data, and cannot be animated. Use a `SpriteRef` if you need
/// animations.
#[derive(Debug)]
pub struct Sprite {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

/// Drawables can be blitted to the pixel buffer and animated.
pub trait Drawable {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixels(&self) -> &[u8];
}

impl Sprite {
    pub fn new(assets: &Assets, asset: Asset) -> Sprite {
        let (width, height, pixels) = assets
            .sprites()
            .get(&asset)
            .unwrap_or_else(|| panic!("pixel data for {:?} is missing!", asset));

        Sprite {
            width: *width,
            height: *height,
            pixels: pixels.to_vec(),
        }
    }
}

impl Drawable for Sprite {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

/// Blit a drawable to the pixel buffer.
pub fn blit<S>(frame: &mut [u8], screen_width: usize, screen_height: usize, dest: &Pos2, sprite: &S)
where
    S: Drawable,
{
    let (dest_x, dest_y) = (dest.x as usize, dest.y as usize);
    assert!(
        dest_x + sprite.width() <= screen_width,
        "can't blit, outside x bounds"
    );
    assert!(
        dest_y + sprite.height() <= screen_height,
        "can't blit, outside y bounds"
    );

    let pixels = sprite.pixels();
    let width = sprite.width() * 4;

    let mut s = 0;
    for y in 0..sprite.height() {
        let i = dest_x * 4 + dest_y * screen_width * 4 + y * screen_width * 4;

        // Merge pixels from sprite into screen
        let zipped = frame[i..i + width].iter_mut().zip(&pixels[s..s + width]);
        for (left, &right) in zipped {
            if right > 0 {
                *left = right;
            }
        }

        s += width;
    }
}

/// Draw a line to the pixel buffer using Bresenham's algorithm.
pub fn line(
    frame: &mut [u8],
    screen_width: usize,
    screen_height: usize,
    p1: &Pos2,
    p2: &Pos2,
    color: [u8; 4],
) {
    let p1 = (p1.x as i64, p1.y as i64);
    let p2 = (p2.x as i64, p2.y as i64);

    for (x, y) in Bresenham::new(p1, p2) {
        let x = usize::min(x as usize, screen_width - 1);
        let y = usize::min(y as usize, screen_height - 1);
        let i = x * 4 + y * screen_width * 4;

        frame[i..i + 4].copy_from_slice(&color);
    }
}

/// Draw a rectangle to the pixel buffer using two points in opposite corners.
pub fn rect(
    frame: &mut [u8],
    screen_width: usize,
    screen_height: usize,
    p1: &Pos2,
    p2: &Pos2,
    color: [u8; 4],
) {
    let p2 = Pos2::new(p2.x - 1.0, p2.y - 1.0);
    let p3 = Pos2::new(p1.x, p2.y);
    let p4 = Pos2::new(p2.x, p1.y);

    line(frame, screen_width, screen_height, p1, &p3, color);
    line(frame, screen_width, screen_height, &p3, &p2, color);
    line(frame, screen_width, screen_height, &p2, &p4, color);
    line(frame, screen_width, screen_height, &p4, p1, color);
}
