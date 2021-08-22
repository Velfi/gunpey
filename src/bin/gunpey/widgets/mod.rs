mod game_board;

use crate::app_state::{AppState, View};
use druid::{
    widget::{Button, Flex, Label, ViewSwitcher, WidgetExt},
    Env, Widget,
};
use std::sync::Arc;

pub fn root() -> impl Widget<AppState> {
    ViewSwitcher::new(
        |data: &AppState, _env| data.current_view(),
        |selector, _data, _env| match selector {
            View::Start => Box::new(start_screen()),
            View::Game => Box::new(game_screen()),
        },
    )
}

pub fn start_screen() -> impl Widget<AppState> {
    let label = Label::new("Gunpey")
        .with_text_size(40.0)
        .padding(10.0)
        .center();

    let start_game_button = Button::new("New Game")
        .on_click(|_ctx, data: &mut AppState, _env: &Env| data.set_current_view(View::Game))
        .padding(5.0);

    Flex::column()
        .with_child(label)
        .with_child(start_game_button)
}

pub fn game_screen() -> impl Widget<AppState> {
    let score = game_score_widget();
    // let game_grid = game_board::widget();

    let new_row_button = Button::new("Add new row")
        .on_click(|_ctx, data: &mut AppState, _env: &Env| {
            data.cycle_grid_rows();
        })
        .padding(5.0);

    let score_button = Button::new("Score a test point")
        .on_click(|_ctx, data: &mut AppState, _env: &Env| {
            Arc::make_mut(&mut data.test).inner += 1;
        })
        .padding(5.0);

    let back_button = Button::new("Back to main menu")
        .on_click(|_ctx, data: &mut AppState, _env: &Env| {
            data.reset_score();
            data.set_current_view(View::Start);
        })
        .padding(5.0);

    Flex::column()
        .with_child(score)
        .with_child(game_board::make_widget())
        .with_child(score_button)
        .with_child(new_row_button)
        .with_child(back_button)
}

fn game_score_widget() -> impl Widget<AppState> {
    let score_label = Label::new("test.inner:");
    let score_counter = Label::new(|data: &AppState, _env: &Env| data.test.inner.to_string());

    Flex::row()
        .with_child(score_label)
        .with_default_spacer()
        .with_child(score_counter)
}
