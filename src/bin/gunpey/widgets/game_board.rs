use crate::{app_state::AppState, assets};
use druid::{
    widget::{prelude::*, Flex, SizedBox, WidgetExt},
    Widget,
};
use druid::{Point, TimerToken};
use gunpey_lib::{cell::Cell, grid::Grid, grid_pos::GridPos, line_fragment::LineFragmentKind};
use log::{debug, trace};
use std::time::{Duration, Instant};

fn build_widget(app_state: &AppState) -> Box<dyn Widget<AppState>> {
    let mut container = Flex::column();
    app_state
        .grid
        .cell_rows_in_render_order()
        .into_iter()
        .for_each(|row| {
            let mut child = Flex::row();
            for cell in row {
                let cell = match cell {
                    Cell::Filled(line_fragment) => {
                        debug!("creating new cell from {:?}", line_fragment);
                        match (line_fragment.is_active, line_fragment.kind) {
                            (true, LineFragmentKind::Caret) => assets::active_caret(),
                            (false, LineFragmentKind::Caret) => assets::caret(),
                            (true, LineFragmentKind::InvertedCaret) => {
                                assets::active_inverted_caret()
                            }
                            (false, LineFragmentKind::InvertedCaret) => assets::inverted_caret(),
                            (true, LineFragmentKind::LeftSlash) => assets::active_left_slash(),
                            (false, LineFragmentKind::LeftSlash) => assets::left_slash(),
                            (true, LineFragmentKind::RightSlash) => assets::active_right_slash(),
                            (false, LineFragmentKind::RightSlash) => assets::right_slash(),
                        }
                    }
                    Cell::Empty => assets::empty_cell(),
                };
                let cell = SizedBox::new(cell).fix_width(52.0).fix_height(32.0);
                child.add_child(cell)
            }

            container.add_child(child);
        });

    container.boxed()
}

struct GameBoardWidget {
    width: usize,
    height: usize,
    timer_id: TimerToken,
    cell_size: Size,
    last_update: Instant,
    children: Box<dyn Widget<AppState>>,
}

impl GameBoardWidget {
    fn rebuild_inner(&mut self, data: &AppState) {
        self.children = build_widget(&data);
    }

    fn grid_pos(&self, p: Point) -> Option<GridPos> {
        let w0 = self.cell_size.width;
        let h0 = self.cell_size.height;
        if p.x < 0.0 || p.y < 0.0 || w0 == 0.0 || h0 == 0.0 {
            return None;
        }
        let x = (p.x / w0) as usize;
        // Grid is rendered top to bottom so flipping the y value is necessary
        let y = (self.height - 1) - (p.y / h0) as usize;
        if y >= self.height || x >= self.width {
            return None;
        }

        let gp = GridPos { x, y };

        Some(gp)
    }

    fn cursor_pos(&self, grid: &Grid, p: Point) -> Option<(GridPos, GridPos)> {
        self.grid_pos(p)
            .map(|a_pos| {
                let b_pos = if a_pos.y == grid.height - 1 {
                    grid.below(a_pos)
                } else {
                    grid.above(a_pos)
                };

                b_pos.map(|b_pos| (a_pos, b_pos))
            })
            .flatten()
    }
}

impl Widget<AppState> for GameBoardWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        match event {
            Event::WindowConnected => {
                ctx.request_paint();
                let deadline = Duration::from_millis(data.iter_interval());
                self.last_update = Instant::now();
                self.timer_id = ctx.request_timer(deadline);
            }
            Event::Timer(id) => {
                if *id == self.timer_id {
                    let deadline = Duration::from_millis(data.iter_interval());
                    self.last_update = Instant::now();
                    self.timer_id = ctx.request_timer(deadline);
                }
            }
            Event::MouseDown(e) => {
                if let Some((grid_pos_a, grid_pos_b)) = self.cursor_pos(&data.grid, e.pos) {
                    data.swap_cells(grid_pos_a, grid_pos_b);
                }
            }
            Event::MouseUp(e) => {
                if let Some(grid_pos) = self.grid_pos(e.pos) {
                    debug!("MouseUp event at {:?}", grid_pos);
                }
            }
            Event::MouseMove(e) => {
                if let Some(grid_pos) = self.grid_pos(e.pos) {
                    trace!("MouseMove event at {:?}", grid_pos);
                }
            }
            _ => {}
        }

        self.children.event(ctx, event, data, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.rebuild_inner(data);
        }
        self.children.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppState, data: &AppState, env: &Env) {
        if !old_data.grid.same(&data.grid) {
            self.rebuild_inner(data);
            ctx.children_changed();
        } else {
            self.children.update(ctx, old_data, data, env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &AppState,
        env: &Env,
    ) -> Size {
        self.children.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        self.children.paint(ctx, data, env)
    }

    fn id(&self) -> Option<WidgetId> {
        self.children.id()
    }
}

pub fn make_widget() -> impl Widget<AppState> {
    GameBoardWidget {
        timer_id: TimerToken::INVALID,
        cell_size: Size {
            width: 52.0,
            height: 32.0,
        },
        height: 10,
        width: 5,
        last_update: Instant::now(),
        children: SizedBox::empty().boxed(),
    }
}
