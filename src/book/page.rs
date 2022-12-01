use crate::book::epub_text::EpubText;
use crate::book::page_element::PageElement;
use druid::im::Vector;
use druid::text::RichText;
use druid::widget::ListIter;
use druid::{ArcStr, Data, ImageBuf, Lens};
use unicode_segmentation::UnicodeSegmentation;

const MAX_PAGE_LINES: usize = 38;

#[derive(Default, Clone, Data, Lens, Debug)]
pub struct Page {
    page: Vector<PageElement>,
    num_lines: usize,
}

impl Page {
    pub(crate) fn new() -> Self {
        Self {
            page: Vector::new(),
            num_lines: 0,
        }
    }
    pub fn add_lines(&mut self, text: &EpubText) -> Result<(), ()> {
        let text_estimated_lines = (text.get_text().graphemes(true).count() / 100) + 1;
        if (*self).num_lines != 0 && ((text_estimated_lines + (*self).num_lines) > MAX_PAGE_LINES) {
            return Err(());
        };
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
        self.page.push_back(PageElement::Text(rich_text));
        (*self).num_lines += text_estimated_lines;
        Ok(())
    }
    pub fn add_image(&mut self, img_data: &[u8]) {
        self.page.push_back(match ImageBuf::from_data(img_data) {
            Ok(im) => {
                //TODO: fare una stima migliore
                self.num_lines += MAX_PAGE_LINES / 4;
                PageElement::Image(im)
            }
            Err(_) => PageElement::Text(RichText::new(ArcStr::from("[Error rendering image]"))),
        });
    }
}


impl ListIter<PageElement> for Page {
    fn for_each(&self, cb: impl FnMut(&PageElement, usize)) {
        self.page.for_each(cb);
    }

    fn for_each_mut(&mut self, cb: impl FnMut(&mut PageElement, usize)) {
        self.page.for_each_mut(cb);
    }

    fn data_len(&self) -> usize {
        self.page.len()
    }
}
