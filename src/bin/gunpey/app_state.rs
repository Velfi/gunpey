use druid::{Data, Lens};
use gunpey_lib::{grid::Grid, new_small_grid};
use rand::prelude::StdRng;
use rand::SeedableRng;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum View {
    Start,
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Test {
    pub inner: u32,
}

impl Test {
    pub fn new(inner: u32) -> Self {
        Self { inner }
    }
}

#[derive(Clone, Data)]
pub struct AppState {
    pub grid: Grid,
    pub rng: Arc<StdRng>,
    pub test: Arc<Test>,
    score: usize,
    current_view: View,
    pub updates_per_second: f64,
}

impl AppState {
    pub fn new() -> Self {
        let grid = new_small_grid();
        let test = Arc::new(Test::new(0));
        let rng = Arc::new(SeedableRng::from_entropy());

        Self {
            current_view: View::Start,
            grid,
            rng,
            score: 0,
            test,
            updates_per_second: 60.0,
        }
    }

    pub fn score(&self) -> usize {
        self.score
    }

    pub fn score_points(&mut self, points: usize) {
        self.score += points;
    }

    pub fn reset_score(&mut self) {
        self.score = 0;
    }

    /// Set the app state's current view.
    pub fn set_current_view(&mut self, view: View) {
        self.current_view = view;
    }

    /// Get a reference to the app state's current view.
    pub fn current_view(&self) -> View {
        self.current_view
    }

    pub fn iter_interval(&self) -> u64 {
        (1000. / self.updates_per_second) as u64
    }
}

pub struct GridLens;

impl Lens<AppState, Grid> for GridLens {
    fn with<V, F: FnOnce(&Grid) -> V>(&self, data: &AppState, f: F) -> V {
        f(&data.grid)
    }

    fn with_mut<V, F: FnOnce(&mut Grid) -> V>(&self, data: &mut AppState, f: F) -> V {
        f(&mut data.grid)
    }
}
