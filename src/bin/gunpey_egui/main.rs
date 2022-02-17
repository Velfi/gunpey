#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::convert::TryInto;
use std::sync::Arc;

use crate::gui::Gui;
use assets::{Asset, Assets};
use egui::{Pos2, Rect};
use gunpey_lib::grid_pos::GridPos;
use gunpey_lib::{cell::Cell, grid::Grid, line_fragment::LineFragmentKind};
use gunpey_lib::{new_random_row, new_small_grid, NewRowGenerationParams};
use log::{debug, error, trace};
use pixels::{Error, Pixels, SurfaceTexture};
use rand::prelude::*;
use sprite::{blit, rect, Sprite};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod assets;
// mod button;
mod gui;
mod sprite;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const GAME_WIDTH: u32 = 320;
const GAME_HEIGHT: u32 = 256;
const BOX_SIZE: i16 = 64;
const CELL_SIZE: usize = 16;

/// Representation of the application state. In this example, a box will bounce around the screen.
pub struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
    // If Some, mouse pointer is over the screen,
    // If None, mouse pointer is outside the screen
    mouse_coordinates: Option<MouseCoordinates>,
    assets: Assets,
    grid: Grid,
    rng: Arc<StdRng>,
}

fn main() -> Result<(), Error> {
    let _ = dotenv::dotenv();
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Gunpey")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let (mut pixels, mut gui) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(GAME_WIDTH, GAME_HEIGHT, surface_texture)?;
        let gui = Gui::new(window_size.width, window_size.height, scale_factor, &pixels);

        (pixels, gui)
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Update egui inputs
        gui.handle_event(&event);

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            // Draw the world
            world.draw(pixels.get_frame());

            // Prepare egui
            gui.prepare(&mut world);

            // Render everything together
            let render_result = pixels.render_with(|encoder, render_target, context| {
                // Render the world texture
                context.scaling_renderer.render(encoder, render_target);

                // Render egui
                gui.render(encoder, render_target, context);

                Ok(())
            });

            // Basic error handling
            if render_result
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Update the scale factor
            if let Some(scale_factor) = input.scale_factor() {
                gui.scale_factor(scale_factor);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                gui.resize(size.width, size.height);
            }

            world.mouse_coordinates = input.mouse().map(|xy| {
                let world_space = pixels
                    .window_pos_to_pixel(xy)
                    .ok()
                    .map(|(x, y)| (x as i16, y as i16));

                let grid_space = world_space
                    .and_then(|(x, y)| {
                        world.world_space_pos_to_grid_space_pos(Pos2::new(x as f32, y as f32))
                    })
                    .map(|grid_pos| grid_pos.to_tuple());

                MouseCoordinates {
                    screen_space: xy,
                    world_space,
                    grid_space,
                }
            });

            if input.mouse_pressed(0) {
                if let Some((cell_pos_a, cell_pos_b)) = world
                    .mouse_coordinates
                    .and_then(|coords| coords.grid_space)
                    .and_then(|(x, y)| world.cursor_pos(GridPos::new(x, y)))
                {
                    match world.grid.swap_cells(cell_pos_a, cell_pos_b) {
                        Ok(_) => {
                            debug!("swapped {} with {}", cell_pos_a, cell_pos_b)
                        }
                        Err(e) => error!("can't swap, {}", e),
                    }
                }
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        let rng = Arc::new(SeedableRng::from_entropy());
        let grid = new_small_grid();
        let assets = assets::load_assets();

        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
            mouse_coordinates: None,
            assets,
            rng,
            grid,
        }
    }

    pub fn swap_cells(&mut self, grid_pos_a: GridPos, grid_pos_b: GridPos) {
        debug!("Swapping tiles at {} and {}", grid_pos_a, grid_pos_b);

        match self.grid.swap_cells(grid_pos_a, grid_pos_b) {
            Ok(_) => self.grid.recalculate_active_cells(),
            Err(err) => error!("Couldn't swap: {}", err),
        };
    }

    pub fn cycle_grid_rows(&mut self) {
        let new_row_params = NewRowGenerationParams {
            width: self.grid.width,
        };

        let popped_row = self.grid.pop_top_row();
        trace!("popped row: {:#?}", popped_row);

        let rng = Arc::make_mut(&mut self.rng);
        let new_row = new_random_row(rng, new_row_params);
        match self.grid.push_bottom_row(new_row) {
            Ok(_) => self.grid.recalculate_active_cells(),
            Err(err) => error!("failed push_row_to_bottom_and_pop_row_from_top: {}", err),
        };
    }

    pub fn reset_grid(&mut self) {
        self.grid = Grid::new(5, 10);
    }

    fn game_grid_rect(&self) -> Rect {
        let game_grid_width = self.grid.width * CELL_SIZE;
        let game_grid_height = self.grid.height * CELL_SIZE;

        Rect::from_x_y_ranges(
            0.0..=(game_grid_width as f32),
            0.0..=(game_grid_height as f32),
        )
    }

    fn world_space_pos_to_grid_space_pos(&self, p: Pos2) -> Option<GridPos> {
        let mut game_grid_rect = self.game_grid_rect();
        let screen_rect =
            Rect::from_x_y_ranges(0.0..=(GAME_WIDTH as f32), 0.0..=(GAME_HEIGHT as f32));
        game_grid_rect.set_center(screen_rect.center());

        let x = ((p.x - game_grid_rect.left()) / CELL_SIZE as f32) as isize;
        let y = ((game_grid_rect.bottom() - p.y) / CELL_SIZE as f32) as isize;

        if x < 0 || x >= self.grid.width as isize || y < 0 || y >= self.grid.height as isize {
            None
        } else {
            Some(GridPos { x, y })
        }
    }

    fn cursor_pos(&self, a_pos: GridPos) -> Option<(GridPos, GridPos)> {
        let b_pos = if a_pos.y == self.grid.height as isize - 1 {
            self.grid.below(a_pos)
        } else {
            self.grid.above(a_pos)
        };

        b_pos.map(|b_pos| (a_pos, b_pos))
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > GAME_WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > GAME_HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % GAME_WIDTH as usize) as i16;
            let y = (i / GAME_WIDTH as usize) as i16;

            if let Some(xy) = self.mouse_coordinates.and_then(|coords| coords.world_space) {
                if xy == (x, y) {
                    pixel.copy_from_slice(&[0xff, 0x00, 0x00, 0xff]);
                    continue;
                }
            }

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }

        let mut game_grid_rect = self.game_grid_rect().expand(1.0);
        let screen_rect =
            Rect::from_x_y_ranges(0.0..=(GAME_WIDTH as f32), 0.0..=(GAME_HEIGHT as f32));
        game_grid_rect.set_center(screen_rect.center());

        let mut row_index = 0;
        let (x_origin, y_origin) = (
            game_grid_rect.left() as usize + 1,
            game_grid_rect.top() as usize + 1,
        );

        let cursor_pos = self
            .mouse_coordinates
            .and_then(|coords| coords.grid_space)
            .and_then(|p| self.cursor_pos(p.into()));

        self.grid
            .cell_rows_in_render_order()
            .into_iter()
            .for_each(|row| {
                let y = row_index * CELL_SIZE + y_origin;

                for (cell_index, cell) in row.into_iter().enumerate() {
                    let x = cell_index * CELL_SIZE + x_origin;
                    let sprite = match cell {
                        Cell::Filled(line_fragment) => {
                            match (line_fragment.is_active, line_fragment.kind) {
                                (true, LineFragmentKind::Caret) => Asset::ActiveCaret,
                                (false, LineFragmentKind::Caret) => Asset::Caret,
                                (true, LineFragmentKind::InvertedCaret) => {
                                    Asset::ActiveInvertedCaret
                                }
                                (false, LineFragmentKind::InvertedCaret) => Asset::InvertedCaret,
                                (true, LineFragmentKind::LeftSlash) => Asset::ActiveLeftSlash,
                                (false, LineFragmentKind::LeftSlash) => Asset::LeftSlash,
                                (true, LineFragmentKind::RightSlash) => Asset::ActiveRightSlash,
                                (false, LineFragmentKind::RightSlash) => Asset::RightSlash,
                            }
                        }
                        Cell::Empty => Asset::EmptyCell,
                    };

                    blit(
                        frame,
                        GAME_WIDTH as usize,
                        GAME_HEIGHT as usize,
                        &Pos2::new(x as f32, y as f32),
                        &Sprite::new(&self.assets, sprite),
                    );

                    if let Some((a_pos, b_pos)) = cursor_pos {
                        let flip = (game_grid_rect.center().y
                            + (CELL_SIZE * self.grid.height) as f32)
                            / 2.0;
                        let top_x = (a_pos.x as f32 * CELL_SIZE as f32) + x_origin as f32;
                        let top_y = flip - (a_pos.y as f32 * CELL_SIZE as f32) + y_origin as f32;
                        let bottom_x = (b_pos.x as f32 * CELL_SIZE as f32) + x_origin as f32;
                        let bottom_y = flip - (b_pos.y as f32 * CELL_SIZE as f32) + y_origin as f32;

                        blit(
                            frame,
                            GAME_WIDTH as usize,
                            GAME_HEIGHT as usize,
                            &Pos2::new(top_x, top_y),
                            &Sprite::new(&self.assets, Asset::Cursor),
                        );

                        blit(
                            frame,
                            GAME_WIDTH as usize,
                            GAME_HEIGHT as usize,
                            &Pos2::new(bottom_x, bottom_y),
                            &Sprite::new(&self.assets, Asset::Cursor),
                        );
                    }
                }
                row_index += 1;
            });

        rect(
            frame,
            GAME_WIDTH.try_into().unwrap(),
            GAME_HEIGHT.try_into().unwrap(),
            &game_grid_rect.left_top(),
            &game_grid_rect.right_bottom(),
            [0xFF, 0x66, 0x00, 0xFF],
        );
    }
}

#[derive(Clone, Copy)]
pub struct MouseCoordinates {
    pub screen_space: (f32, f32),
    pub world_space: Option<(i16, i16)>,
    pub grid_space: Option<(isize, isize)>,
}
