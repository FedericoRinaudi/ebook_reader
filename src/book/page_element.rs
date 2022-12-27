use crate::book::epub_text::EpubText;
use druid::piet::{PietTextLayoutBuilder, TextStorage as PietTextStorage};
use druid::text::{EnvUpdateCtx, Link, RichText, TextStorage};
use druid::{ArcStr, Data, Env, ImageBuf};




#[derive(Clone, Data, Debug)]
pub enum PageElement {
    Text(EpubText),
    Image(ImageBuf),
    Error(EpubText),
}


impl PageElement {

    /* Crea un PageElement a partire da un'immagine */
    pub fn _from_image(img_data: &[u8]) -> Self {
        match ImageBuf::from_data(img_data) {
            Ok(im) => {
                // println!("Immagine caricata con successo!");
                PageElement::Image(im)
            }
            Err(_) => {
                // println!("Errore, interrotto");
                PageElement::Text(EpubText::from("[IMG]".to_string()))
            }
        }
    }

    pub fn is_err(&self) -> bool {
        matches!(*self, PageElement::Error(_))
    }
}

//TODO gestisco l'enum con le lens
impl PietTextStorage for PageElement {
    //
    fn as_str(&self) -> &str {
        match self {
            PageElement::Text(t) => &t.text,
            PageElement::Image(_) => "[IMG]",
            PageElement::Error(e) => &e.text,
        }
    }
}

impl TextStorage for PageElement {
    fn add_attributes(&self, builder: PietTextLayoutBuilder, env: &Env) -> PietTextLayoutBuilder {
        match self {
            PageElement::Text(t) => t.to_richtext().add_attributes(builder, env),
            PageElement::Image(_) => RichText::new("".into()).add_attributes(builder, env),
            PageElement::Error(e) => e.to_richtext().add_attributes(builder, env),
        }
    }

    fn env_update(&self, ctx: &EnvUpdateCtx) -> bool {
        match self {
            PageElement::Text(t) => t.to_richtext().env_update(ctx),
            PageElement::Image(_) => true,
            PageElement::Error(e) => e.to_richtext().env_update(ctx),
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
