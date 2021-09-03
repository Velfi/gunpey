#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::convert::TryInto;
use std::sync::Arc;

use crate::gui::Gui;
use assets::{Asset, Assets};
use egui::{Pos2, Rect};
use gunpey_lib::grid_pos::GridPos;
use gunpey_lib::{cell::Cell, grid::Grid, line_fragment::LineFragmentKind};
use gunpey_lib::{new_random_row, NewRowGenerationParams};
use log::{debug, error};
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
mod button;
mod gui;
mod sprite;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 256;
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
    mouse_xy: Option<(i16, i16)>,
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
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
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
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
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

            world.mouse_xy = input
                .mouse()
                .map(|xy| pixels.window_pos_to_pixel(xy).ok())
                .flatten()
                // Ignore any rust-analyzer errors here, they're fake
                .map(|(x, y)| (x as i16, y as i16));

            if input.mouse_pressed(0) {
                if let Some((cell_pos_a, cell_pos_b)) = world
                    .mouse_xy
                    .and_then(|(x, y)| world.cursor_pos(Pos2::new(x as f32, y as f32)))
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
        let grid = Grid::new_from_str(
            r#"
        .cc
        .l.
        ..l
        "#,
        );
        let assets = assets::load_assets();

        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
            mouse_xy: None,
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
        debug!("popped row: {:#?}", popped_row);

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

    fn grid_pos(&self, p: Pos2) -> Option<GridPos> {
        let game_grid_rect = self.game_grid_rect();

        if p.x < game_grid_rect.left() || p.y < game_grid_rect.top() {
            return None;
        }

        let x = (p.x / CELL_SIZE as f32) as usize;
        let y = (p.y / CELL_SIZE as f32) as usize;

        if y >= game_grid_rect.bottom() as usize || x >= game_grid_rect.right() as usize {
            None
        } else {
            Some(GridPos { x, y })
        }
    }

    fn cursor_pos(&self, p: Pos2) -> Option<(GridPos, GridPos)> {
        self.grid_pos(p)
            .map(|a_pos| {
                let b_pos = if a_pos.y == self.grid.height - 1 {
                    self.grid.below(a_pos)
                } else {
                    self.grid.above(a_pos)
                };

                b_pos.map(|b_pos| (a_pos, b_pos))
            })
            .flatten()
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
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
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            if let Some(xy) = self.mouse_xy {
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
        let screen_rect = Rect::from_x_y_ranges(0.0..=(WIDTH as f32), 0.0..=(HEIGHT as f32));
        game_grid_rect.set_center(screen_rect.center());

        let mut row_index = 0;
        let (x_origin, y_origin) = (
            game_grid_rect.left() as usize + 1,
            game_grid_rect.top() as usize + 1,
        );

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
                        WIDTH as usize,
                        HEIGHT as usize,
                        &Pos2::new(x as f32, y as f32),
                        &Sprite::new(&self.assets, sprite),
                    );
                }
                row_index += 1;
            });

        rect(
            frame,
            WIDTH.try_into().unwrap(),
            HEIGHT.try_into().unwrap(),
            &game_grid_rect.left_top(),
            &game_grid_rect.right_bottom(),
            [0xFF, 0x66, 0x00, 0xFF],
        );
    }
}