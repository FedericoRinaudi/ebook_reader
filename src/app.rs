use crate::book::chapter::Chapter;
use crate::bookcase::{BookCase, BookInfo};
use crate::ocr::find_ch;
use crate::utilities::is_part;
use crate::view::view::View;
use crate::Book;
use druid::{im::HashSet, im::Vector, Data, ExtEventSink, Lens, Selector, Target};
use std::thread;

pub const TRIGGER_ON: Selector<()> = Selector::new("wrapper.focus_on");
pub const TRIGGER_OFF: Selector<()> = Selector::new("wrapper.focus_off");
pub const SCROLL_REQUEST: Selector<()> = Selector::new("wrapper.scroll");
pub const FINISH_SLOW_FUNCTION: Selector<Option<(usize, usize, String)>> =
    Selector::new("finish_slow_function");
pub const FINISH_LEPTO_LOAD: Selector<Option<String>> = Selector::new("leptonica.finish_load");
pub const FINISH_BOOK_LOAD: Selector<Option<Book>> = Selector::new("book.finish_load");

#[derive(Clone, Data, PartialEq, Copy)]
pub enum InputMode {
    OcrJump,
    EbookAdd,
    OcrSyn0,
    OcrSyn1,
    None,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::None
    }
}

#[derive(Default, Clone, Data, Lens)]
pub struct ApplicationState {
    pub error_message: Option<String>,
    pub book_to_view: Book,
    pub edit: bool,
    // Serve a switchare da view mode a edit mode
    pub xml_backup: String,
    // xml backup useful to discard changes done in edit mode
    pub modified: HashSet<usize>,
    //find better solution
    pub view: View,
    pub bookcase: BookCase,
    pub is_loading: bool,
    pub i_mode: InputMode,
    pub book_to_align: Book,
}

impl ApplicationState {
    pub fn new() -> ApplicationState {
        let app = ApplicationState {
            error_message: None,
            book_to_view: Book::empty_book(),
            edit: false,
            xml_backup: "".to_string(),
            modified: HashSet::new(),
            view: View::new(),
            bookcase: BookCase::new(),
            is_loading: false,
            i_mode: InputMode::None,
            book_to_align: Book::empty_book(),
        };
        //app.update_view();
        app
    }

    pub fn update_view(&mut self) {
        self.view
            .update_view(self.book_to_view.format_current_chapter());
        let ocr = self.get_current_book_info().ocr;
        if ocr.is_aligned() {
            let _ = self.view.guess_lines(
                ocr.get_avg_ch(),
                ocr.get_first_page_lines(),
                ocr.get_other_page_lines(),
                self.book_to_view
                    .chapters
                    .get(self.book_to_view.get_ch())
                    .unwrap_or(&Chapter::default())
                    .initial_page,
            );
        }
    }

    pub fn get_library(&self) -> &Vector<BookInfo> {
        &(*self).bookcase.library
    }

    pub fn close_current_book(&mut self) {
        for book_info in self.bookcase.library.iter_mut() {
            if book_info.get_path().to_str().unwrap() == self.book_to_view.get_path() {
                book_info.start_chapter = self.book_to_view.get_nav().get_ch();
                book_info.start_element_number = self.book_to_view.get_nav().get_element_numer();
                break;
            }
        }
        self.bookcase.update_meta();
        self.book_to_view = Book::empty_book();
    }

    pub fn set_book_to_read(&mut self, book: Book) {
        self.book_to_view = book;
        //self.is_loading = false;
    }

    pub fn set_book_to_align(&mut self, book: Book) {
        self.book_to_align = book;
    }

    pub fn map_pages(&mut self, overwrite: bool) -> Result<(), usize> {
        //Return number of uncertain page as Err to ask for additional user input, lowprio TODO

        let chapters = self.book_to_align.chapters.clone();
        let book_info = self.get_mut_current_book_info().unwrap();
        let ocr = book_info.ocr.clone();
        if overwrite {
            book_info.mapped_pages = Vector::new();
        }
        let mut page = ocr.get_mapping(ocr.first.unwrap()).unwrap().page;
        let mut temp_view = View::new();
        for (id, ch) in chapters.iter().enumerate() {
            if id + 1 <= book_info.mapped_pages.len() {
                continue;
            }
            if ocr.first_chap.unwrap() <= id {
                temp_view.update_view(ch.format().clone());
                if is_part(temp_view.current_view.clone()) {
                    if page % 2 == 0 {
                        page += 1;
                        book_info.mapped_pages.push_back(page);
                        page += 2;
                    } else {
                        book_info.mapped_pages.push_back(page);
                        page += 2;
                    }
                } else {
                    book_info.mapped_pages.push_back(page);
                    page += temp_view
                        .guess_lines(
                            ocr.get_avg_ch(),
                            ocr.get_first_page_lines(),
                            ocr.get_other_page_lines(),
                            0,
                        )
                        .unwrap();
                }
            } else {
                book_info.mapped_pages.push_back(0);
            }
        }

        println!(
            "VETTORE DI FIRST_PAGES: {:?}",
            book_info.mapped_pages.clone()
        );
        return Ok(());
    }

    pub fn get_current_book_info(&self) -> BookInfo {
        let path = if !self.book_to_view.is_empty() {
            self.book_to_view.get_path()
        } else if !self.book_to_align.is_empty() {
            self.book_to_align.get_path()
        } else {
            return BookInfo::default();
        };
        if let Some(res) = self.bookcase.library.iter().find(|b| *b.path == path) {
            (*res).clone()
        } else {
            BookInfo::default()
        }
    }

    pub fn get_mut_current_book_info(&mut self) -> Option<&mut BookInfo> {
        let path = if !self.book_to_view.is_empty() {
            self.book_to_view.get_path()
        } else if !self.book_to_align.is_empty() {
            self.book_to_align.get_path()
        } else {
            return None;
        };

        if let Some(res) = self
            .bookcase
            .library
            .iter_mut()
            .find(|b| (**b).path == path)
        {
            return Some(res);
        }

        None
    }

    pub fn ocr_jump(&mut self, sink: ExtEventSink, str: String) {
        match self.i_mode {
            InputMode::OcrJump => th_find(str.clone(), sink, self.book_to_view.chapters.clone()),
            InputMode::OcrSyn1 | InputMode::OcrSyn0 => {
                th_find(str.clone(), sink, self.book_to_align.chapters.clone())
            }
            _ => (),
        }
    }
}

fn th_find(str: String, sink: ExtEventSink, vec: Vector<Chapter>) {
    thread::spawn(move || match find_ch(str.clone(), vec) {
        Some((index, offset)) => sink
            .submit_command(
                FINISH_SLOW_FUNCTION,
                Option::Some((index, offset, str)),
                Target::Auto,
            )
            .expect("command failed to submit"),
        None => {
            sink.submit_command(FINISH_SLOW_FUNCTION, Option::None, Target::Auto)
                .expect("command failed to submit");
        }
    });
}
