use crate::book::epub_text::EpubText;
use druid::piet::{PietTextLayoutBuilder, TextStorage as PietTextStorage};
use druid::text::{EnvUpdateCtx, RichText, TextStorage};
use druid::{Data, Env, ImageBuf};

#[derive(Clone, Data, Debug)]
pub struct PageElement {
    pub content: ContentType,
    pub size: Option<(f64, f64)>,
    //#[data(ignore)]
    pub pg_offset: usize,
    pub not_in_html: bool
}

impl PageElement {
    pub fn new(con: ContentType, not_in_html: bool) -> PageElement {
        PageElement {
            content: con,
            size: Option::None,
            pg_offset: 0,
            not_in_html
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
pub enum ContentType {
    Text(EpubText),
    Image(ImageBuf),
    Error(EpubText),
}

impl ContentType {
    /* Crea un PageElement a partire da un'immagine */
    pub fn _from_image(img_data: &[u8]) -> Self {
        match ImageBuf::from_data(img_data) {
            Ok(im) => {
                // println!("Immagine caricata con successo!");
                ContentType::Image(im)
            }
            Err(_) => {
                // println!("Errore, interrotto");
                ContentType::Text(EpubText::from("[IMG]".to_string()))
            }
        }
    }

    pub fn is_err(&self) -> bool {
        matches!(*self, ContentType::Error(_))
    }
}

/*
//TODO gestisco l'enum con le lens
impl PietTextStorage for ContentType {
    //
    fn as_str(&self) -> &str {
        match self {
            ContentType::Text(t) => &t.text,
            ContentType::Image(_) => "[IMG]",
            ContentType::Error(e) => &e.text,
        }
    }
}

impl TextStorage for ContentType {
    fn add_attributes(&self, builder: PietTextLayoutBuilder, env: &Env) -> PietTextLayoutBuilder {
        match self {
            ContentType::Text(t) => t.to_richtext().add_attributes(builder, env),
            ContentType::Image(_) => RichText::new("".into()).add_attributes(builder, env),
            ContentType::Error(e) => e.to_richtext().add_attributes(builder, env),
        }
    }

    fn env_update(&self, ctx: &EnvUpdateCtx) -> bool {
        match self {
            ContentType::Text(t) => t.to_richtext().env_update(ctx),
            ContentType::Image(_) => true,
            ContentType::Error(e) => e.to_richtext().env_update(ctx),
        }
    }
    /*
    fn links(&self) -> &[Link] {
        match self {
            PageElement::Text(t) => t.to_richtext().links().clone(),
            PageElement::Image(_) => Default::default(),
            PageElement::Error(e) => e.to_richtext().links().clone(),
        }
    }
    */
}
*/

impl PartialEq for ContentType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ContentType::Text(s1), ContentType::Text(s2)) => s1.text == s2.text,
            (ContentType::Error(e1), ContentType::Error(e2)) => e1.text == e2.text,
            (ContentType::Image(_i1), ContentType::Image(_i2)) => true, //TODO: IMPLEMENT IMG COMPARISON
            // Return false if the enums contain different types
            _ => false,
        }
    }
}
