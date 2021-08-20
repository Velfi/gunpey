mod app_state;
mod assets;
mod widgets;

use app_state::AppState;
use druid::theme::WINDOW_BACKGROUND_COLOR;
use druid::{AppLauncher, Color, LocalizedString, WindowDesc};
use widgets::root;

const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Gunpey");

fn main() {
    let _ = dotenv::dotenv();
    env_logger::init();
    let main_window = WindowDesc::new(root())
        .title(WINDOW_TITLE)
        .window_size((800.0, 600.0));
    let data = AppState::new();
    AppLauncher::with_window(main_window)
        .configure_env(|env, _state| {
            // Change window background to a lighter gray
            env.set(WINDOW_BACKGROUND_COLOR, Color::rgb8(0x85, 0x85, 0x85))
        })
        .log_to_console()
        .launch(data)
        .expect("Failed to launch application");
}

// use gunpey_lib::{new_random_row, new_small_grid, NewRowGenerationParams};
// use rand::prelude::StdRng;
// use rand::SeedableRng;

// fn main() {
//     let mut grid = new_small_grid();
//     let mut rng: StdRng = SeedableRng::from_entropy();

//     for _ in 0..3 {
//         let new_row = new_random_row(
//             &mut rng,
//             NewRowGenerationParams {
//                 width: grid.width(),
//             },
//         );
//         let _ = grid.push_row_to_bottom_and_pop_row_from_top(new_row);
//     }

//     println!("{}", grid);
// }
