use druid::commands::CLOSE_WINDOW;
use druid::widget::prelude::*;
use druid::widget::{Controller, ControllerHost, Label, LabelText};
use druid::{
    Color, Data, Point, TimerToken, Vec2, Widget, WidgetExt, WindowConfig, WindowHandle, WindowId,
    WindowLevel, WindowSizePolicy,
};
use druid::{InternalLifeCycle, Rect, Scalable, Screen};
use std::time::{Duration, Instant};

const TOOLTIP_DELAY: Duration = Duration::from_millis(150);
const TOOLTIP_DELAY_CHECK: Duration = Duration::from_millis(120);
const TOOLTIP_BORDER_COLOR: Color = Color::GREEN;
const TOOLTIP_BORDER_WIDTH: f64 = 1.0;
const TOOLTIP_OFFSET: Vec2 = Vec2::new(15.0, 15.0);

#[derive(Clone)]
pub(crate) enum TooltipState {
    Off,
    Waiting {
        timer: TimerToken,
        last_mouse_move: Instant,
        last_mouse_pos: Point,
    },
    Showing {
        id: WindowId,
        // We store last_mouse_pos here because we seem to sometimes get a synthesized MouseMove
        // event after showing the tooltip (maybe because the mouse leaves the window?). By storing
        // the last mouse position, we can filter out these spurious moves.
        last_mouse_pos: Point,
    },
}

pub struct TooltipCtrl<T> {
    pub(crate) text: LabelText<T>,
    pub(crate) state: TooltipState,
    pub(crate) show_if_click: bool,
}

impl<T: Data, W: Widget<T>> Controller<T, W> for TooltipCtrl<T> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, ev: &Event, data: &mut T, env: &Env) {
        self.state = match self.state {
            TooltipState::Waiting {
                timer,
                last_mouse_move,
                last_mouse_pos,
            } => match ev {
                Event::MouseMove(ev) if ctx.is_hot() => TooltipState::Waiting {
                    timer,
                    last_mouse_move: Instant::now(),
                    last_mouse_pos: ev.window_pos,
                },
                Event::MouseUp(_) | Event::MouseMove(_) | Event::Wheel(_) => TooltipState::Off,
                Event::Timer(tok) if tok == &timer => {
                    ctx.set_handled();
                    let elapsed = Instant::now().duration_since(last_mouse_move);
                    if elapsed > TOOLTIP_DELAY_CHECK {
                        self.text.resolve(data, env);
                        let tooltip_position_in_window_coordinates =
                            last_mouse_pos + TOOLTIP_OFFSET;
                        let win_id = ctx.new_sub_window(
                            WindowConfig::default()
                                .show_titlebar(false)
                                .window_size_policy(WindowSizePolicy::Content)
                                .set_level(WindowLevel::Tooltip(ctx.window().clone()))
                                .set_position(tooltip_position_in_window_coordinates),
                            // FIXME: we'd like to use the actual label text instead of
                            // resolving, but LabelText isn't Clone
                            Label::new(self.text.display_text())
                                .border(TOOLTIP_BORDER_COLOR, TOOLTIP_BORDER_WIDTH)
                                .on_monitor(ctx.window()),
                            data.clone(),
                            env.clone(),
                        );
                        TooltipState::Showing {
                            id: win_id,
                            last_mouse_pos,
                        }
                    } else {
                        TooltipState::Waiting {
                            timer: ctx.request_timer(TOOLTIP_DELAY - elapsed),
                            last_mouse_move,
                            last_mouse_pos,
                        }
                    }
                }
                _ => self.state.clone(),
            },
            TooltipState::Off => match ev {
                Event::MouseMove(ev) if ctx.is_hot() && !self.show_if_click => {
                    TooltipState::Waiting {
                        timer: ctx.request_timer(TOOLTIP_DELAY),
                        last_mouse_move: Instant::now(),
                        last_mouse_pos: ev.window_pos,
                    }
                }
                Event::MouseDown(ev) if self.show_if_click => TooltipState::Waiting {
                    timer: ctx.request_timer(TOOLTIP_DELAY),
                    last_mouse_move: Instant::now(),
                    last_mouse_pos: ev.window_pos,
                },
                _ => TooltipState::Off,
            },
            TooltipState::Showing { id, last_mouse_pos } => match ev {
                Event::MouseMove(ev) if ctx.is_hot() => {
                    // This is annoying. On GTK, after showing a window we instantly get a new
                    // MouseMove event, with a mouse position that tends to be slightly different
                    // than the previous one. If we don't test the positions, this causes the
                    // tooltip to immediately close.
                    if (ev.window_pos - last_mouse_pos).hypot2() > 1.0 {
                        ctx.submit_command(CLOSE_WINDOW.to(id));
                        TooltipState::Waiting {
                            timer: ctx.request_timer(TOOLTIP_DELAY),
                            last_mouse_move: Instant::now(),
                            last_mouse_pos: ev.window_pos,
                        }
                    } else {
                        self.state.clone()
                    }
                }
                Event::MouseMove(_) | Event::Wheel(_) => {
                    ctx.submit_command(CLOSE_WINDOW.to(id));
                    self.state.clone()
                }
                Event::MouseDown(_) | Event::MouseUp(_) if !self.show_if_click => {
                    ctx.submit_command(CLOSE_WINDOW.to(id));
                    self.state.clone()
                }
                Event::MouseUp(_) if self.show_if_click => {
                    ctx.submit_command(CLOSE_WINDOW.to(id));
                    TooltipState::Off
                }
                _ => self.state.clone(),
            },
        };
        child.event(ctx, ev, data, env);
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        ev: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(false) = ev {
            if let TooltipState::Showing { id, .. } = self.state {
                ctx.submit_command(CLOSE_WINDOW.to(id));
                self.state = TooltipState::Off;
            }
        }
        child.lifecycle(ctx, ev, data, env);
    }
}

pub struct OnMonitor<W> {
    pub(crate) inner: W,
    pub(crate) parent: WindowHandle,
}

fn screen_bounds(w: &WindowHandle) -> Rect {
    let monitors = Screen::get_monitors();
    let scale = w.get_scale().unwrap_or_default();
    let window_origin = w.get_position();
    for m in monitors {
        if m.virtual_rect().to_dp(scale).contains(window_origin) {
            return m.virtual_work_rect().to_dp(scale);
        }
    }
    Rect::from_origin_size(Point::ZERO, Size::new(f64::INFINITY, f64::INFINITY))
}

fn calc_nudge(rect: Rect, bounds: Rect) -> Vec2 {
    // Returns an offset that tries to translate interval to within bounds.
    fn nudge(interval: (f64, f64), bounds: (f64, f64)) -> f64 {
        let nudge_up = (bounds.0 - interval.0).max(0.0);
        let nudge_down = (bounds.1 - interval.1).min(0.0);
        if nudge_up > 0.0 {
            nudge_up
        } else {
            nudge_down
        }
    }

    let x_nudge = nudge((rect.x0, rect.x1), (bounds.x0, bounds.x1));
    let y_nudge = nudge((rect.y0, rect.y1), (bounds.y0, bounds.y1));
    Vec2::new(x_nudge, y_nudge)
}

impl<T: Data, W: Widget<T>> Widget<T> for OnMonitor<W> {
    fn event(&mut self, ctx: &mut EventCtx, ev: &Event, data: &mut T, env: &Env) {
        self.inner.event(ctx, ev, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, ev: &LifeCycle, data: &T, env: &Env) {
        match ev {
            LifeCycle::Size(_) | LifeCycle::Internal(InternalLifeCycle::ParentWindowOrigin) => {
                let w = ctx.window();
                let rect = Rect::from_origin_size(ctx.window_origin(), ctx.size());
                let current_window_pos = w.get_position();
                let bounds = screen_bounds(&self.parent);
                let nudge = calc_nudge(rect + current_window_pos.to_vec2(), bounds);
                w.set_position(current_window_pos + nudge);
            }
            _ => {}
        }
        self.inner.lifecycle(ctx, ev, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.inner.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.inner.paint(ctx, data, env);
    }
}

pub trait TipExt<T: Data>: Widget<T> + Sized + 'static {
    /// Open a tooltip when the mouse is hovered over this widget.
    fn tooltip<LT: Into<LabelText<T>>>(
        self,
        text: LT,
        show_if_click: bool,
    ) -> ControllerHost<Self, TooltipCtrl<T>> {
        self.controller(TooltipCtrl {
            text: text.into(),
            state: TooltipState::Off,
            show_if_click,
        })
    }

    /// A convenience method for ensuring that this widget is fully visible on the same monitor as
    /// some other window.
    fn on_monitor(self, parent: &WindowHandle) -> OnMonitor<Self> {
        OnMonitor {
            inner: self,
            parent: parent.clone(),
        }
    }
}

impl<T: Data, W: Widget<T> + 'static> TipExt<T> for W {}
