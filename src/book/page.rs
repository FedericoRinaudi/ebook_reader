use druid::im::Vector;
use druid::text::RichText;
use druid::widget::ListIter;
use unicode_segmentation::UnicodeSegmentation;
use druid::{Data,Lens};
use crate::book::epub_text::EpubText;

const MAX_PAGE_LINES: usize = 42;
#[derive(Clone, Data, Lens)]
pub struct Page {
    page: Vector<RichText>,
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
        self.page.push_back(rich_text);
        (*self).num_lines += text_estimated_lines;
        Ok(())
    }
}


impl ListIter<RichText> for Page {
    fn for_each(&self, cb: impl FnMut(&RichText, usize)) {
        self.page.for_each(cb);
    }

    fn for_each_mut(&mut self, cb: impl FnMut(&mut RichText, usize)) {
        self.page.for_each_mut(cb);
    }

    fn data_len(&self) -> usize {
        self.page.data_len()
    }
}
