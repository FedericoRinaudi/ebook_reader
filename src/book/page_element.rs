use crate::book::epub_text::EpubText;
use crate::utilities::th_load_image;
use druid::piet::{PietTextLayoutBuilder, TextStorage as PietTextStorage};
use druid::text::{EnvUpdateCtx, RichText, TextStorage};
use druid::{Data, Env, ExtEventSink, ImageBuf};

#[derive(Clone, Data, Debug)]
pub struct PageElement {
    pub content: ContentType,
    pub size: Option<(f64, f64)>,
    //#[data(ignore)]
    pub pg_offset: (usize, bool),
    pub not_in_html: bool,
}

impl PageElement {
    pub fn from_text(con: EpubText, not_in_html: bool) -> PageElement {
        PageElement {
            content: ContentType::Text(con),
            size: Option::None,
            pg_offset: (0, false),
            not_in_html,
        }
    }
    pub fn from_img_async(
        con: ImageState,
        not_in_html: bool,
        sink: ExtEventSink,
        epub_path: String,
    ) -> PageElement {
        if let ImageState::Waiting(buf) = con.clone() {
            /**/
            th_load_image(sink, buf.clone(), epub_path);
        }
        PageElement {
            content: ContentType::Image(con),
            size: None,
            pg_offset: (0, false),
            not_in_html,
        }
    }

    pub fn from_img_sync(con: ImageState, not_in_html: bool) -> PageElement {
        PageElement {
            content: ContentType::Image(con),
            size: None,
            pg_offset: (0, false),
            not_in_html,
        }
    }
    pub fn from_error(con: EpubText, not_in_html: bool) -> PageElement {
        PageElement {
            content: ContentType::Error(con),
            size: None,
            pg_offset: (0, false),
            not_in_html,
        }
    }
}

impl PietTextStorage for PageElement {
    //
    fn as_str(&self) -> &str {
        match &self.content {
            ContentType::Text(t) => &t.text,
            ContentType::Image(_) => "[IMG]",
            ContentType::Error(e) => &e.text,
        }
    }
}

impl TextStorage for PageElement {
    fn add_attributes(&self, builder: PietTextLayoutBuilder, env: &Env) -> PietTextLayoutBuilder {
        match &self.content {
            ContentType::Text(t) => t.to_richtext().add_attributes(builder, env),
            ContentType::Image(_) => RichText::new("".into()).add_attributes(builder, env),
            ContentType::Error(e) => e.to_richtext().add_attributes(builder, env),
        }
    }

    fn env_update(&self, ctx: &EnvUpdateCtx) -> bool {
        match &self.content {
            ContentType::Text(t) => t.to_richtext().env_update(ctx),
            ContentType::Image(_) => true,
            ContentType::Error(e) => e.to_richtext().env_update(ctx),
        }
    }
}

#[derive(Clone, Data, Debug)]
pub enum ImageState {
    Present(ImageBuf),
    Waiting(String),
}

#[derive(Clone, Data, Debug)]
pub enum ContentType {
    Text(EpubText),
    Image(ImageState),
    Error(EpubText),
}

impl ContentType {
    pub fn is_err(&self) -> bool {
        matches!(*self, ContentType::Error(_))
    }
}

impl PartialEq for ContentType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ContentType::Text(s1), ContentType::Text(s2)) => s1.text == s2.text,
            (ContentType::Error(e1), ContentType::Error(e2)) => e1.text == e2.text,
            (ContentType::Image(i1), ContentType::Image(i2)) => i1 == i2,
            // Return false if the enums contain different types
            _ => false,
        }
    }
}

impl PartialEq for ImageState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ImageState::Present(_), ImageState::Present(_)) => true,
            (ImageState::Waiting(i1), ImageState::Waiting(i2)) => i1 == i2,
            _ => false,
        }
    }
}
