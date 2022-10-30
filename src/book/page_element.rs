use druid::piet::{PietTextLayoutBuilder, TextStorage as PietTextStorage};
use druid::text::{EnvUpdateCtx, Link, RichText, TextStorage};
use druid::{Data, Env, ImageBuf};

#[derive(Clone, Data)]
pub enum PageElement {
    Text(RichText),
    Image(ImageBuf)
}

impl PietTextStorage for PageElement {
    fn as_str(&self) -> &str {
        match self {
            PageElement::Text(t) => t.as_str(),
            PageElement::Image(_) => "[IMG]"
        }
    }
}


impl TextStorage for PageElement {
    fn add_attributes(
        &self,
        builder: PietTextLayoutBuilder,
        env: &Env,
    ) -> PietTextLayoutBuilder {
        match self {
            PageElement::Text(t) => t.add_attributes(builder, env),
            PageElement::Image(_) => RichText::new("".into()).add_attributes(builder, env)
        }
    }

    fn env_update(&self, ctx: &EnvUpdateCtx) -> bool {
        match self {
            PageElement::Text(t) => t.env_update(ctx),
            PageElement::Image(_) => true
        }
    }

    fn links(&self) -> &[Link] {
        match self {
            PageElement::Text(t) => t.links(),
            PageElement::Image(_) => Default::default()
        }
    }
}
