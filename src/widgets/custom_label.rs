use crate::book::page_element::PageElement;
use druid::widget::{LineBreaking, RawLabel};
use druid::{
    BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Selector, Size, UpdateCtx, Widget,
};

pub const UPDATE_SIZE: Selector<()> = Selector::new("label.size_changed");

pub struct BetterLabel {
    child: RawLabel<PageElement>,
}

impl BetterLabel {
    pub fn new() -> BetterLabel {
        let mut rawlab = RawLabel::new();
        rawlab.set_line_break_mode(LineBreaking::WordWrap);
        BetterLabel { child: rawlab }
    }
}

impl Widget<PageElement> for BetterLabel {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut PageElement, env: &Env) {
        self.child.event(ctx, event, data, env);
        match event {
            Event::Command(cmd) => {
                if cmd.get(UPDATE_SIZE).is_some() {
                    data.size = Some(<(f64, f64)>::from(ctx.size()));
                    //ctx.submit_command(SCROLL_REQUEST);
                }
            }
            Event::MouseDown(_e) => {
                self.child.set_text_color(Color::GRAY);
                ctx.request_layout();
                ctx.request_paint();
            }
            Event::MouseUp(_e) => {
                self.child.set_text_color(Color::WHITE);
                ctx.request_layout();
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &PageElement,
        env: &Env,
    ) {
        match event {
            _ => {}
        }
        self.child.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &PageElement,
        data: &PageElement,
        env: &Env,
    ) {
        self.child.update(ctx, old_data, &data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &PageElement,
        env: &Env,
    ) -> Size {
        let size = self.child.layout(ctx, bc, data, env);
        //ctx.submit_command(UPDATE_SIZE);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &PageElement, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}
