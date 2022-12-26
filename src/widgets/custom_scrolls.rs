use crate::app::{ApplicationState, TRIGGER_OFF, TRIGGER_ON};
use druid::widget::{Axis, Scroll};
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx, Widget,
};

pub struct BetterScroll<W: Widget<ApplicationState>> {
    child: Scroll<ApplicationState, W>,
}

impl<W: Widget<ApplicationState>> BetterScroll<W> {
    pub fn new(widget: W) -> Self {
        BetterScroll {
            child: Scroll::new(widget).vertical(),
        }
    }
}

impl<W: Widget<ApplicationState>> Widget<ApplicationState> for BetterScroll<W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut ApplicationState, env: &Env) {
        self.child.event(ctx, event, data, env);
        match event {
            Event::WindowCloseRequested => {
                if data.modified.len() > 0 || data.edit {
                    println!("Window close not implemented for unsaved edits/edit mode")
                } else {
                    data.current_book
                        .get_mut_nav()
                        .set_line(self.child.offset_for_axis(Axis::Vertical));
                    data.close_current_book();
                }
            }
            Event::Command(cmd) => {
                if cmd.get(TRIGGER_ON).is_some() {
                    //println!("Triggered on to {}", self.child.offset_for_axis(Axis::Vertical));
                    self.child
                        .scroll_to_on_axis(Axis::Vertical, data.current_book.get_nav().get_line());
                    ctx.request_paint();
                } else if cmd.get(TRIGGER_OFF).is_some() {
                    //println!("Triggered off to {} out of {}", self.child.offset_for_axis(Axis::Vertical), self.child.child_size().height);
                    data.current_book
                        .get_mut_nav()
                        .set_line(self.child.offset_for_axis(Axis::Vertical));
                    data.view.scroll_height = self.child.child_size().height;
                }
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &ApplicationState,
        env: &Env,
    ) {
        match event {
            LifeCycle::HotChanged(false) => {
                ctx.submit_command(TRIGGER_OFF);
                //ctx.submit_command(TRIGGER_SYN)
            }
            _ => {}
        }
        self.child.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &ApplicationState,
        data: &ApplicationState,
        env: &Env,
    ) {
        self.child.update(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &ApplicationState,
        env: &Env,
    ) -> Size {
        let size = self.child.layout(ctx, bc, data, env);
        self.child
            .scroll_to_on_axis(Axis::Vertical, data.current_book.get_nav().get_line());
        //println!("Layed to {}", data.current_book.get_nav().get_line());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ApplicationState, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}

pub struct SyncScroll<W: Widget<ApplicationState>> {
    child: Scroll<ApplicationState, W>,
    flag: bool,
}

impl<W: Widget<ApplicationState>> SyncScroll<W> {
    pub fn new(widget: W) -> Self {
        SyncScroll {
            child: Scroll::new(widget).vertical(),
            flag: true,
        }
    }
}

impl<W: Widget<ApplicationState>> Widget<ApplicationState> for SyncScroll<W> {
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

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &ApplicationState,
        env: &Env,
    ) {
        self.child.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &ApplicationState,
        data: &ApplicationState,
        env: &Env,
    ) {
        self.child.update(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &ApplicationState,
        env: &Env,
    ) -> Size {
        let size = self.child.layout(ctx, bc, data, env);

        if self.flag {
            let rate = data.view.scroll_height / self.child.child_size().height;
            self.child.scroll_to_on_axis(
                Axis::Vertical,
                data.current_book.get_nav().get_line() * rate + 15.0,
            );
            self.flag = false
        }
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ApplicationState, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}
