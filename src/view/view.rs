use crate::book::page_element::ImageState::Present;
use crate::book::page_element::PageElement;
use crate::{ApplicationState, ContentType};
use druid::{im::Vector, Data, Lens, LocalizedString};
use unicode_segmentation::UnicodeSegmentation;

pub const WINDOW_TITLE: LocalizedString<ApplicationState> =
    LocalizedString::new("Ebook Reader");
const VIEW_SIZE: (f64, f64) = (800.0, 800.0);
const EDIT_SIZE: (f64, f64) = (1600.0, 800.0);
const HOME_SIZE: (f64, f64) = (800.0, 800.0);

#[derive(Default, Clone, Data, Lens)]
pub struct View {
    window_size_view: (f64, f64),
    window_size_edit: (f64, f64),
    window_size_home: (f64, f64),
    pub current_view: Vector<PageElement>,
    pub scroll_height: f64,
    pub ocr_form_stage: usize,
}

impl View {
    pub fn new() -> Self {
        View {
            window_size_view: VIEW_SIZE,
            window_size_edit: EDIT_SIZE,
            window_size_home: HOME_SIZE,
            current_view: Vector::new(),
            scroll_height: 0.0,
            ocr_form_stage: 1,
        }
    }

    pub fn update_view(&mut self, vec: Vector<PageElement>) {
        self.current_view = vec;
    }

    pub fn get_window_size_view(&self) -> (f64, f64) {
        self.window_size_view
    }

    pub fn get_window_size_edit(&self) -> (f64, f64) {
        self.window_size_edit
    }

    pub fn get_window_size_home(&self) -> (f64, f64) {
        self.window_size_home
    }

    pub fn set_window_size_view(&mut self, size: (f64, f64)) {
        (*self).window_size_view = size
    }

    pub fn set_window_size_edit(&mut self, size: (f64, f64)) {
        (*self).window_size_edit = size
    }

    pub fn _set_window_size_home(&mut self, size: (f64, f64)) {
        (*self).window_size_home = size
    }

    pub fn get_element_offset(&self, n: usize) -> f64 {
        let mut sum = 0.0;
        for el in self.current_view.iter().take(if n == 0 { 0 } else { n }) {
            sum += el.size.unwrap_or((0.0, 0.0)).1;
        }
        sum
    }

    pub fn get_element_from_offset(&self, height: f64) -> usize {
        let mut element_number = 0;
        let mut sum = 0.0;

        for cont in self.current_view.iter() {
            if let Some(size) = cont.size {
                if size.1 + sum <= height {
                    sum += size.1;
                    element_number += 1;
                } else {
                    break;
                }
            }
        }
        element_number
    }

    pub fn ocr_offset_to_element(&self, mut offset: usize) -> usize {
        // A partire da un offset di words>5 trova il page element
        let mut page_element_number = 0;
        for page_element in &self.current_view {
            if let ContentType::Text(text) = page_element.content.clone() {
                let long_words = text
                    .text
                    .split(" ")
                    .map(|el| el.chars().filter(|c| c.is_alphabetic()).collect::<String>())
                    .filter(|w| w.len() >= 5)
                    .count();
                if (offset as i32 - long_words as i32) < 0 {
                    break;
                } else {
                    offset -= long_words;
                }
            }
            page_element_number += 1;
        }
        page_element_number
    }

    pub fn guess_lines(
        &mut self,
        max_chars: f64,
        first: usize,
        second: usize,
        starting_page: usize,
    ) -> Result<usize, ()> {
        let mut guessed_lines = 0;
        let mut curr_page = 1;

        for el in self.current_view.iter_mut() {
            if el.not_in_html {
                continue;
            }
            if let ContentType::Text(text) = el.clone().content {
                let mut element_lines =
                    (text.text.trim().graphemes(true).count() as f64 / max_chars).ceil() as usize;
                if element_lines == 0 {
                    element_lines = 1;
                } else if text.text.trim().graphemes(true).count() % (max_chars as usize) <= 3 {
                    element_lines -= 1;
                }

                let max_lines = if curr_page == 1 { first } else { second };
                guessed_lines = if (guessed_lines + element_lines) <= max_lines {
                    guessed_lines + element_lines
                } else {
                    curr_page += 1;
                    el.pg_offset.1 = true;
                    guessed_lines + element_lines - max_lines
                };
            } else if let ContentType::Image(img_buf) = el.clone().content {
                if let Present(img_buf) = img_buf {
                    let element_lines = img_buf.height() / 20;
                    let max_lines = if curr_page == 1 { first } else { second };
                    guessed_lines = if (guessed_lines + element_lines) <= max_lines {
                        guessed_lines + element_lines
                    } else {
                        curr_page += 1;
                        element_lines
                    };
                }
            }

            el.pg_offset.0 = if starting_page == 0 {
                0
            } else {
                curr_page + starting_page - 1
            }
        }

        Ok(curr_page)
    }
}
