use std::borrow::Borrow;
use druid::widget::{Controller, Scroll};
use druid::{Data, Env, Event, EventCtx, Widget};

pub struct Update<T> {
    /// A closure that will be invoked when the child widget is clicked.
    action: Box<dyn Fn(&mut EventCtx, &mut T, &Env)>,
}

impl<T: Data> Update<T> {
    /// Create a new clickable [`Controller`] widget.
    pub fn new(action: impl Fn(&mut EventCtx, &mut T, &Env) + 'static) -> Self {
        Update {
            action: Box::new(action),
        }
    }
}

impl<T: Data, W: Widget<T>> Controller<T, W> for Update<T> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::KeyUp(_event) => {
                (self.action)(ctx, data, env);
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
}

pub struct Wheel<T> {
    /// A closure that will be invoked when the child widget is clicked.
    action: Box<dyn Fn(&mut EventCtx, &mut T, &Env)>,
}

impl<T: Data> Wheel<T> {
    /// Create a new clickable [`Controller`] widget.
    pub fn new(action: impl Fn(&mut EventCtx, &mut T, &Env) + 'static) -> Self {
        Wheel {
            action: Box::new(action),
        }
    }
}

impl<T: Data, W: Widget<T>> Controller<T, Scroll<T, W>> for Wheel<T> {
    fn event(&mut self, child: &mut Scroll<T, W>, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::Wheel(_event) => {

                println!("{}", child.offset())
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
}


