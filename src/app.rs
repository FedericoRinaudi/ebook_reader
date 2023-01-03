use std::thread;
use crate::bookcase::{BookCase, BookInfo};
use crate::Book;
use druid::{im::HashSet, im::Vector, Data, Lens, Selector, ExtEventSink, Target};
use crate::book::chapter::Chapter;
use crate::ocr::{find_ch, Mapping, OcrData};

use crate::view::view::View;

//pub const TRIGGER_A: Selector<()> = Selector::new("monitor.update_status");
pub const TRIGGER_ON: Selector<()> = Selector::new("wrapper.focus_on");
pub const TRIGGER_OFF: Selector<()> = Selector::new("wrapper.focus_off");
pub const SCROLL_REQUEST: Selector<()> = Selector::new("wrapper.scroll");
//pub const TRIGGER_SYN: Selector<()> = Selector::new("wrapper.focus_syn");
pub const FINISH_SLOW_FUNCTION: Selector<Option<(usize, usize)>> =
    Selector::new("finish_slow_function");
pub const FINISH_LEPTO_LOAD: Selector<Option<String>> =
    Selector::new("leptonica.finish_load");

#[derive(Clone, Data, PartialEq, Copy)]
pub enum InputMode {
    OcrJump,
    EbookAdd,
    OcrSyn0(usize),
    OcrSyn1(usize),
    None,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::None
    }
}

impl InputMode {
    pub fn index(&self) -> Option<usize> {
        match &self {
            InputMode::OcrSyn0(id) | InputMode::OcrSyn1(id) => Some(*id),
            _ => None
        }
    }
}




#[derive(Default, Clone, Data, Lens)]
pub struct ApplicationState {
    pub error_message: Option<String>,
    pub current_book: Book,
    pub edit: bool,
    // Serve a switchare da view mode a edit mode
    pub xml_backup: String,
    // xml backup useful to discard changes done in edit mode
    pub modified: HashSet<usize>,
    //find better solution
    pub view: View,
    pub bookcase: BookCase,
    pub is_loading: bool,
    pub i_mode: InputMode
}

impl ApplicationState {
    pub fn new() -> ApplicationState {
        let app = ApplicationState {
            error_message: None,
            current_book: Book::empty_book(),
            edit: false,
            xml_backup: "".to_string(),
            modified: HashSet::new(),
            view: View::new(),
            bookcase: BookCase::new(),
            is_loading: false,
            i_mode: InputMode::None
        };
        //app.update_view();
        app
    }

    pub fn update_view(&mut self) {
        self.view
            .update_view(self.current_book.format_current_chapter());
    }

    pub fn get_library(&self) -> &Vector<BookInfo> {
        &(*self).bookcase.library
    }

    pub fn close_current_book(&mut self) {
        for book_info in self.bookcase.library.iter_mut() {
            if book_info.get_path().to_str().unwrap() == self.current_book.get_path() {
                book_info.start_chapter = self.current_book.get_nav().get_ch();
                book_info.start_element_number = self.current_book.get_nav().get_element_numer();
                break;
            }
        }
        self.bookcase.update();
        self.current_book = Book::empty_book();
    }

    pub fn set_book(&mut self, book: Book){
        self.current_book = book;
        if self.get_current().mapped_pages.len() == 0 && self.get_current().ocr.is_aligned() {
            self.map_pages()
        }
    }

    fn map_pages(&mut self){

        let ocr = self.get_current().ocr;

        let mut v = Vector::new();
        let mut page = ocr.get_mapping(ocr.first.unwrap()).unwrap().page;
        let mut temp_view = View::new();
        for (id, ch) in self.current_book.chapters.iter_mut().enumerate() {
            if ocr.first_chap.unwrap() <= id {
                temp_view.update_view(ch.format().clone());
                if ch.is_part {
                    if page % 2 == 0 {
                        page += 1;
                        v.push_back(page);
                        page += 2;
                    } else {
                        v.push_back( page);
                        page += 2;
                    }
                } else {
                    v.push_back(page);
                    page += temp_view.guess_lines(ocr.get_avg_ch(), ocr.get_first_page_lines(), ocr.get_other_page_lines());
                }
            } else {
                v.push_back(0);
            }
        }

        println!("VETTORE DI FIRST_PAGES: {:?}", v.clone());
        self.get_mut_current().unwrap().mapped_pages = v;
    }


    pub fn get_current(&self) -> BookInfo {

        return if let Some(index) = self.i_mode.index() {
            (&self).bookcase.library[index].clone()
        } else {
            if let Some(res) = &self
                .bookcase
                .library
                .iter()
                .find(|b| b.path == *(&self.current_book.get_path()))
            {
                (*res).clone()
            } else {
                BookInfo {
                    name: "default".to_string(),
                    path: "".to_string(),
                    start_chapter: 0,
                    start_element_number: 0,
                    cover_path: "".to_string(),
                    ocr: OcrData::new(),
                    mapped_pages: Vector::new()
                }
            }
        }
    }

    pub fn get_mut_current(&mut self) -> Option<&mut BookInfo> {

        if let Some(index) = self.i_mode.index() {
            return Some(&mut self.bookcase.library[index])
        } else {
            if let Some(res) = self
                .bookcase
                .library
                .iter_mut()
                .find(|b| b.path == *(&self.current_book.get_path()))
            {
                return Some(res);
            }
        }
        None
    }

    pub fn ocr_jump(&mut self, sink: ExtEventSink, str: String, log:bool) {
        th_find(
            str.clone(),
            sink,
            self.current_book.chapters.clone(),
        );
        if log {
            self.get_mut_current().unwrap().ocr.ocr_log(str.clone());
        }
    }

}


fn th_find(str: String, sink: ExtEventSink, vec: Vector<Chapter>) {
    thread::spawn(move || {
        match find_ch(str, vec) {
            Some((index, offset)) => {
                sink.submit_command(
                    FINISH_SLOW_FUNCTION,
                    Option::Some((index, offset)),
                    Target::Auto,
                )
                    .expect("command failed to submit")
            }
            None => {
                sink.submit_command(FINISH_SLOW_FUNCTION, Option::None, Target::Auto)
                    .expect("command failed to submit");
            }
        }
    });
}