use crate::book::epub_text::EpubText;
use druid::piet::{PietTextLayoutBuilder, TextStorage as PietTextStorage};
use druid::text::{EnvUpdateCtx, Link, RichText, TextStorage};
use druid::{ArcStr, Data, Env, ImageBuf};

#[derive(Clone, Data, Debug)]
pub enum PageElement {
    Text(RichText),
    Image(ImageBuf),
}
impl PageElement {
    /* Crea un PageElement a partire da un EpubText */
    pub fn from_text(text: &EpubText) -> Self {
        let mut rich_text = RichText::new(text.get_text().as_str().into());
        for range_attributes in text.get_attributes().values() {
            for range_attr in range_attributes {
                match range_attr.get_end() {
                    Some(end) => rich_text.add_attribute(
                        (*range_attr).get_start()..end,
                        range_attr.get_attribute().clone(),
                    ),
                    None => rich_text.add_attribute(
                        (*range_attr).get_start()..,
                        range_attr.get_attribute().clone(),
                    ),
                };
            }
        }
        PageElement::Text(rich_text)
    }

    /* Crea un PageElement a partire da un'immagine */
    pub fn from_image(img_data: &[u8]) -> Self {
        match ImageBuf::from_data(img_data) {
            Ok(im) => {
                // println!("Immagine caricata con successo!");
                PageElement::Image(im)},
            Err(_) => {
                // println!("Errore, interrotto");
                PageElement::Text(RichText::new(ArcStr::from("[Error rendering image]")))
            },
        }
    }
}

//TODO gestisco l'enum con le lens
impl PietTextStorage for PageElement {
    //
    fn as_str(&self) -> &str {
        match self {
            PageElement::Text(t) => t.as_str(),
            PageElement::Image(_) => "[IMG]",
        }
    }
}

impl TextStorage for PageElement {
    fn add_attributes(&self, builder: PietTextLayoutBuilder, env: &Env) -> PietTextLayoutBuilder {
        match self {
            PageElement::Text(t) => t.add_attributes(builder, env),
            PageElement::Image(_) => RichText::new("".into()).add_attributes(builder, env),
        }
    }

    fn env_update(&self, ctx: &EnvUpdateCtx) -> bool {
        match self {
            PageElement::Text(t) => t.env_update(ctx),
            PageElement::Image(_) => true,
        }
    }

    fn links(&self) -> &[Link] {
        match self {
            PageElement::Text(t) => t.links(),
            PageElement::Image(_) => Default::default(),
        }
    }
}
