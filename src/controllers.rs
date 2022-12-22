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


impl<W: Widget<ApplicationState>> Controller<ApplicationState, Scroll<ApplicationState, W>> for ViewWrapper {
    fn event(&mut self, child: &mut Scroll<ApplicationState, W>, ctx: &mut EventCtx, event: &Event, data: &mut ApplicationState, env: &Env) {

        match event {
            Event::WindowCloseRequested => {
                data.current_book.get_mut_nav().set_line(child.offset_for_axis(Axis::Vertical));
                data.close_current_book();
            }
            Event::Command(c, ..) =>{
                if c.get(TRIGGER_ON).is_some(){
                    child.scroll_to_on_axis(Axis::Vertical, data.current_book.get_nav().get_line());
                    ctx.request_paint();
                } else if c.get(TRIGGER_OFF).is_some(){
                    data.current_book.get_mut_nav().set_line(child.offset_for_axis(Axis::Vertical));
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
            LifeCycle::HotChanged(false) => {
                println!("hot out view");
                ctx.submit_command(TRIGGER_OFF);
            }
            _ => {}
        }
        child.lifecycle(ctx, event, data, env);
    }

}

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

    fn lifecycle(&mut self, child: &mut Flex<ApplicationState>, ctx: &mut LifeCycleCtx, event: &LifeCycle,  data: &ApplicationState, env: &Env){

        match event {
            LifeCycle::HotChanged(true) => {
                println!("TRIGGER ON");
                ctx.submit_command(TRIGGER_ON);
            }/*
            LifeCycle::HotChanged(false) => {
                println!("TRIGGER ON 2");
                ctx.submit_command(TRIGGER_ON);
            }*/
            _ => {}
        }

        child.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, child:&mut Flex<ApplicationState>, ctx: &mut UpdateCtx, old_data: &ApplicationState, data: &ApplicationState, env: &Env) {

        child.update(ctx, old_data, data, env);

        if data.edit != old_data.edit {
            println!("UPDATED");
            ctx.submit_command(TRIGGER_ON);
        }

    }
}

