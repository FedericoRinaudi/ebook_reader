use druid::im::Vector;
use druid::text::{EnvUpdateCtx, Link, RichText, TextStorage};
use druid::widget::ListIter;
use unicode_segmentation::UnicodeSegmentation;
use druid::{ArcStr, Data, Env, ImageBuf, Lens};
use druid::piet::{PietTextLayoutBuilder, TextStorage as PietTextStorage};
use crate::book::epub_text::EpubText;

const MAX_PAGE_LINES: usize = 42;


#[derive(Clone, Data)]
pub enum PageElementContent {
    Text(RichText),
    Image(ImageBuf)
}

impl PietTextStorage for PageElementContent {
    fn as_str(&self) -> &str {
        match self {
            PageElementContent::Text(t) => t.as_str(),
            PageElementContent::Image(_) => "[IMG]"
        }
    }
}


impl TextStorage for PageElementContent {
    fn add_attributes(
        &self,
        builder: PietTextLayoutBuilder,
        env: &Env,
    ) -> PietTextLayoutBuilder {
        match self {
            PageElementContent::Text(t) => t.add_attributes(builder, env),
            PageElementContent::Image(_) => RichText::new("".into()).add_attributes(builder, env)
        }
    }

    fn env_update(&self, ctx: &EnvUpdateCtx) -> bool {
        match self {
            PageElementContent::Text(t) => t.env_update(ctx),
            PageElementContent::Image(_) => true
        }
    }

    fn links(&self) -> &[Link] {
        match self {
            PageElementContent::Text(t) => t.links(),
            PageElementContent::Image(_) => Default::default()
        }
    }
}

#[derive(Clone, Data, Lens)]
pub struct Page {
    page: Vector<PageElementContent>,
    num_lines: usize
}

impl Page {
    pub(crate) fn new() -> Self {
        Self {
            page: Vector::new(),
            num_lines: 0
        }
    }
    pub fn add_lines(&mut self, text: &EpubText) -> Result<(), ()> {
        let text_estimated_lines = (text.get_text().graphemes(true).count() / 100) + 1 ;
        if (*self).num_lines != 0 && ((text_estimated_lines + (*self).num_lines) > MAX_PAGE_LINES) {
            return Err(());
        };
        let mut rich_text = RichText::new(text.get_text().as_str().into());
        for range_attributes in text.get_attributes().values(){
            for range_attr in range_attributes{
                match range_attr.get_end() {
                    Some(end) => rich_text.add_attribute((*range_attr).get_start()..end, range_attr.get_attribute().clone()),
                    None => rich_text.add_attribute((*range_attr).get_start().., range_attr.get_attribute().clone()),
                };
            }
        }
        self.page.push_back(PageElementContent::Text(rich_text));
        (*self).num_lines += text_estimated_lines;
        Ok(())
    }
    pub fn add_image(&mut self, img_data: &[u8]) {
        self.page.push_back(match ImageBuf::from_data(img_data){
            Ok(im) => {
                //TODO: faccio una stima migliore, per ora se ho un immagine cambio pagina
                self.num_lines += MAX_PAGE_LINES;
                PageElementContent::Image(im)
            }
            Err(_) => {PageElementContent::Text(RichText::new(ArcStr::from("[Error rendering image]")))}
        });
    }
}


impl ListIter<PageElementContent> for Page {
    fn for_each(&self, cb: impl FnMut(&PageElementContent, usize)) {
        self.page.for_each(cb);
    }

    fn for_each_mut(&mut self, cb: impl FnMut(&mut PageElementContent, usize)) {
        self.page.for_each_mut(cb);
    }

    fn data_len(&self) -> usize {
        self.page.len()
    }
}

