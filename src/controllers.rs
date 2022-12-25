use druid::widget::Controller;
use druid::{Data, Env, Event, EventCtx, Widget};

pub struct Update<T> {
    action: Box<dyn Fn(&mut EventCtx, &mut T, &Env)>,
}

impl<T: Data> Update<T> {
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


/*
pub struct ClickableOpacity <T: Data> {
    action: Box<dyn Fn(&mut EventCtx, &mut T, &Env)>,
}

impl <T: Data> ClickableOpacity<T> {
    pub fn new(on_click: impl Fn(&mut EventCtx, &mut T, &Env) + 'static) -> Self {
        Self {
           action: on_click
        }
    }
}

impl<T: Data> Controller<T, Align<T>> for ClickableOpacity<T> {
    fn event(&mut self, child: &mut Align<T>, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::MouseUp(_) => {
                child.set_opacity(1.);
                (self.action)(ctx, data, env);
            },
            Event::MouseDown(_) => child.set_opacity(0.5),
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
}
*/
