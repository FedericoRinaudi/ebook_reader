use crate::app::{ApplicationState, SCROLL_REQUEST, TRIGGER_OFF, TRIGGER_ON};
use crate::widgets::custom_label::UPDATE_SIZE;
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
                    data.book_to_view.get_mut_nav().set_element_number(
                        data.view
                            .get_element_from_offset(self.child.offset_for_axis(Axis::Vertical)),
                    );
                    data.close_current_book();
                }
            }
            Event::Command(cmd) => {
                if cmd.get(TRIGGER_ON).is_some() {
                    self.child.scroll_to_on_axis(
                        Axis::Vertical,
                        data.view
                            .get_element_offset(data.book_to_view.get_nav().get_element_numer()),
                    );
                    ctx.request_paint();
                } else if cmd.get(TRIGGER_OFF).is_some() {
                    data.book_to_view.get_mut_nav().set_element_number(
                        data.view
                            .get_element_from_offset(self.child.offset_for_axis(Axis::Vertical)),
                    );
                    data.view.scroll_height = self.child.child_size().height;
                } else if cmd.get(SCROLL_REQUEST).is_some() {
                    self.child.scroll_to_on_axis(
                        Axis::Vertical,
                        data.view
                            .get_element_offset(data.book_to_view.get_nav().get_element_numer()),
                    );
                    ctx.request_paint();
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
        if data.view.current_view.iter().any(|a| a.size.is_some())
            && !old_data.view.current_view.iter().any(|a| a.size.is_some())
        {
            self.child.scroll_to_on_axis(
                Axis::Vertical,
                data.view
                    .get_element_offset(data.book_to_view.get_nav().get_element_numer()),
            );
            ctx.request_paint();
        }
        if data
            .view
            .current_view
            .iter()
            .zip(old_data.view.current_view.iter())
            .any(|(a1, a2)| a1.content != a2.content)
        {
            self.child.update(ctx, old_data, data, env)
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &ApplicationState,
        env: &Env,
    ) -> Size {
        let size = self.child.layout(ctx, bc, data, env);
        ctx.submit_command(UPDATE_SIZE);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ApplicationState, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}

pub struct SyncScroll<W: Widget<ApplicationState>> {
    child: Scroll<ApplicationState, W>,
}

impl<W: Widget<ApplicationState>> SyncScroll<W> {
    pub fn new(widget: W) -> Self {
        SyncScroll {
            child: Scroll::new(widget).vertical(),
        }
    }
}

impl<W: Widget<ApplicationState>> Widget<ApplicationState> for SyncScroll<W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut ApplicationState, env: &Env) {
        self.child.event(ctx, event, data, env);
        match event {
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
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ApplicationState, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}
