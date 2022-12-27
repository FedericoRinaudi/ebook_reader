use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size, UpdateCtx, Widget};
use druid::widget::{LineBreaking, RawLabel};
use crate::{ApplicationState, PageElement};

pub struct BetterLabel {
    child: RawLabel<PageElement>,
}

impl BetterLabel {
    /*TODO: FAI CASO PER IMMAGINE*/
    pub fn new() -> BetterLabel {

        let mut rawlab = RawLabel::new();
        rawlab.set_line_break_mode(LineBreaking::WordWrap);
            BetterLabel {
                child: rawlab
            }
    }
}

impl Widget<PageElement> for BetterLabel {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut PageElement, env: &Env) {
        self.child.event(ctx, event, data, env);
        match event {
            Event::MouseDown(_) => println!("Cujerbfkjewnr"),
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
        self.child.update(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &PageElement,
        env: &Env,
    ) -> Size {
        let size = self.child.layout(ctx, bc, data, env);
        println!("{:?}", size);
        //println!("Layed to {}", data.current_book.get_nav().get_line());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &PageElement, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}