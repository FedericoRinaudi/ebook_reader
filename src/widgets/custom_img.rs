use crate::book::page_element::PageElement;
use crate::widgets::custom_label::UPDATE_SIZE;
use druid::widget::{FillStrat, Flex, Image};
use druid::{
    BoxConstraints, Env, Event, EventCtx, ImageBuf, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Size, UpdateCtx, Widget,
};

pub struct BetterImage {
    child: Flex<PageElement>,
}

impl BetterImage {
    /*TODO: FAI CASO PER IMMAGINE*/
    pub fn new(buf: ImageBuf) -> BetterImage {
        let img = Image::new(buf).fill_mode(FillStrat::ScaleDown);
        let row = Flex::row();

        BetterImage {
            child: row.with_child(img),
        }
    }
}

impl Widget<PageElement> for BetterImage {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut PageElement, env: &Env) {
        self.child.event(ctx, event, data, env);
        match event {
            Event::Command(cmd) => {
                if cmd.get(UPDATE_SIZE).is_some() {
                    data.size = Some(<(f64, f64)>::from(ctx.size()));
                    //ctx.submit_command(SCROLL_REQUEST);
                }
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
        //data.size = *size;
        ctx.submit_command(UPDATE_SIZE);
        //
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &PageElement, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}
