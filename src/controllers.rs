use std::borrow::Borrow;
use std::time::Duration;
use druid::widget::{Axis, ClipBox, Controller, Flex, Scroll};
use druid::{Data, Env, Event, EventCtx, Vec2, Widget, LifeCycle, Notification, LifeCycleCtx, UpdateCtx, LayoutCtx, BoxConstraints, Size, PaintCtx, InternalEvent};
use druid::commands::SCROLL_TO_VIEW;
use druid::platform_menus::mac::file::print;
use druid::Selector;
use crate::app::{ApplicationState, TRIGGER_OFF, TRIGGER_ON, TRIGGER_SYN};

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


pub struct BetterScroll<W: Widget<ApplicationState>> {
    child: Scroll<ApplicationState, W>
}

impl <W: Widget<ApplicationState>> BetterScroll<W> {
    pub fn new(widget: W) -> Self {
        BetterScroll {
            child: Scroll::new(widget).vertical(),
        }
    }
}

impl <W: Widget<ApplicationState>> Widget<ApplicationState> for BetterScroll<W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut ApplicationState, env: &Env) {

        self.child.event(ctx, event, data, env);
        match event {
            Event::WindowCloseRequested => {
                data.current_book.get_mut_nav().set_line(self.child.offset_for_axis(Axis::Vertical));
                data.close_current_book();
            }
            Event::Command(cmd) => if cmd.get(TRIGGER_ON).is_some(){
                //println!("Triggered on to {}", self.child.offset_for_axis(Axis::Vertical));
                self.child.scroll_to_on_axis(Axis::Vertical, data.current_book.get_nav().get_line());
                ctx.request_paint();
            }else if cmd.get(TRIGGER_OFF).is_some(){
                //println!("Triggered off to {} out of {}", self.child.offset_for_axis(Axis::Vertical), self.child.child_size().height);
                data.current_book.get_mut_nav().set_line(self.child.offset_for_axis(Axis::Vertical));
                data.view.scroll_height = self.child.child_size().height;
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &ApplicationState, env: &Env) {

        match event {
            LifeCycle::HotChanged(false) => {
                ctx.submit_command(TRIGGER_OFF);
                //ctx.submit_command(TRIGGER_SYN)
            }
            _ => {}
        }
        self.child.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &ApplicationState, data: &ApplicationState, env: &Env) {
        self.child.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &ApplicationState, env: &Env) -> Size {
        let size = self.child.layout(ctx, bc, data, env);
        self.child.scroll_to_on_axis(Axis::Vertical, data.current_book.get_nav().get_line());
        //println!("Layed to {}", data.current_book.get_nav().get_line());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ApplicationState, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}


pub struct SyncScroll<W: Widget<ApplicationState>> {
    child: Scroll<ApplicationState, W>
}

impl <W: Widget<ApplicationState>> SyncScroll<W> {
    pub fn new(widget: W) -> Self {
        SyncScroll {
            child: Scroll::new(widget).vertical(),
        }
    }
}

impl <W: Widget<ApplicationState>> Widget<ApplicationState> for SyncScroll<W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut ApplicationState, env: &Env) {

        self.child.event(ctx, event, data, env);
        match event {
            /*
            Event::Command(cmd) => if cmd.get(TRIGGER_SYN).is_some(){
                println!("Synched on to {}", self.child.offset_for_axis(Axis::Vertical));
                self.child.scroll_to_on_axis(Axis::Vertical, data.current_book.get_nav().get_line());
                ctx.request_paint();
            }
            */
            _ => {}
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &ApplicationState, env: &Env) {
        self.child.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &ApplicationState, data: &ApplicationState, env: &Env) {
        self.child.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &ApplicationState, env: &Env) -> Size {
        let size = self.child.layout(ctx, bc, data, env);
        let rate =  data.view.scroll_height / self.child.child_size().height;
        self.child.scroll_to_on_axis(Axis::Vertical, data.current_book.get_nav().get_line()*rate +15.0);

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ApplicationState, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}


/*
pub struct ViewWrapper {
    action: Box<dyn Fn(&mut EventCtx, &mut ApplicationState, &Env)>,
}

impl ViewWrapper {
    pub fn new(action: impl Fn(&mut EventCtx, &mut ApplicationState, &Env) + 'static) -> Self {
        ViewWrapper {
            action: Box::new(action),
        }
    }

}


impl<W: Widget<ApplicationState>> Controller <ApplicationState, Scroll<ApplicationState, W>> for ViewWrapper {
    fn event(&mut self, child: &mut Scroll<ApplicationState, W>, ctx: &mut EventCtx, event: &Event, data: &mut ApplicationState, env: &Env) {

        child.event(ctx, event, data, env);
        match event {
            Event::WindowCloseRequested => {
                data.current_book.get_mut_nav().set_line(child.offset_for_axis(Axis::Vertical));
                data.close_current_book();
            }
            Event::Command(cmd) if cmd.is(TRIGGER_ON) => {
                    child.scroll_to_on_axis(Axis::Vertical, data.current_book.get_nav().get_line());
                    ctx.request_paint();
            }
            Event::Command(cmd) if cmd.is(TRIGGER_OFF) => {
                data.current_book.get_mut_nav().set_line(child.offset_for_axis(Axis::Vertical));
            }
            _ => {}
        }

    }
    fn lifecycle(&mut self, child: &mut Scroll<ApplicationState, W>, ctx: &mut LifeCycleCtx, event: &LifeCycle,  data: &ApplicationState, env: &Env){

        match event {
            LifeCycle::HotChanged(false) => {
                ctx.submit_command(TRIGGER_OFF);
            }
            _ => {}
        }
        child.lifecycle(ctx, event, data, env);
    }

}*/

/*
pub struct EditWrapper {
    action: Box<dyn Fn(&mut EventCtx, &mut ApplicationState, &Env)>,
}

impl EditWrapper {
    pub fn new(action: impl Fn(&mut EventCtx, &mut ApplicationState, &Env) + 'static) -> Self {
        EditWrapper {
            action: Box::new(action),
        }
    }
}


impl<W: Widget<ApplicationState>> Controller<ApplicationState, Scroll<ApplicationState, W>> for EditWrapper {
    fn event(&mut self, child: &mut Scroll<ApplicationState, W>, ctx: &mut EventCtx, event: &Event, data: &mut ApplicationState, env: &Env) {

        match event {
            Event::Command(c, ..) =>{
                if c.get(TRIGGER_ON).is_some(){
                    child.scroll_to_on_axis(Axis::Vertical, data.current_book.get_nav().get_line());
                    ctx.request_paint();
                }
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
    fn lifecycle(&mut self, child: &mut Scroll<ApplicationState, W>, ctx: &mut LifeCycleCtx, event: &LifeCycle,  data: &ApplicationState, env: &Env){

        match event {
            LifeCycle::HotChanged(true) => {
                println!("hot");
                ctx.submit_command(TRIGGER_ON)
            }
            /*
            LifeCycle::HotChanged(false) => {
                ctx.submit_command(TRIGGER_OFF)
            }
            */
            _ => {}
        }
        child.lifecycle(ctx, event, data, env);
    }

}
*/

/*
pub struct DisplayWrapper {
    action: Box<dyn Fn(&mut EventCtx, &mut ApplicationState, &Env)>,
}

impl DisplayWrapper {
    pub fn new(action: impl Fn(&mut EventCtx, &mut ApplicationState, &Env) + 'static) -> Self {
        DisplayWrapper {
            action: Box::new(action),
        }
    }
}


impl Controller <ApplicationState, Flex<ApplicationState>> for DisplayWrapper {

    fn event(&mut self, child: &mut Flex<ApplicationState>, ctx: &mut EventCtx, event: &Event, data: &mut ApplicationState, env: &Env) {
        child.event(ctx, event, data, env);
        match event {
            Event::WindowConnected=> {
                println!("CONNECTED");
            },
            _ => {}
        }
    }

    fn lifecycle(&mut self, child: &mut Flex<ApplicationState>, ctx: &mut LifeCycleCtx, event: &LifeCycle,  data: &ApplicationState, env: &Env){

        match event {/*
            LifeCycle::HotChanged(true) => {
                ctx.submit_command(TRIGGER_ON);
            }*/
            _ => {}
        }

        child.lifecycle(ctx, event, data, env);
    }

}*/
