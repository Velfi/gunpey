pub mod adjacency;
pub mod cell;
pub mod error;
pub mod grid;
pub mod grid_pos;
pub mod line_fragment;

use cell::Cell;
use druid::im::Vector;
use grid::Grid;
use rand::Rng;

pub struct NewRowGenerationParams {
    pub width: usize,
}

pub fn new_random_row(
    rng: &mut impl Rng,
    // TODO is this really how you destructure params? Feels goofy
    NewRowGenerationParams { width }: NewRowGenerationParams,
) -> Vector<Cell> {
    // TODO make this a param
    // Approximately half of a new row should be filled with line segments. This chooses a proportion close to half.
    let percent_of_row_filled_with_cells = rng.gen_range(40.0..60.0);

    (0..width)
        .into_iter()
        .map(|_| {
            let f = rng.gen::<f32>() * 100.0;
            // Here, we use the proportion we chose earlier an fill out our cells based on that proportion
            if f < percent_of_row_filled_with_cells {
                Cell::Filled(rng.gen())
            } else {
                Cell::Empty
            }
        })
        .collect()
}

pub fn new_small_grid() -> Grid {
    Grid::new(5, 10)
}
