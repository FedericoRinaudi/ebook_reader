use crate::bookcase::{BookCase, BookInfo};
use crate::Book;
use druid::{im::HashSet, im::Vector, Data, Lens, Selector};

use crate::view::view::View;

//pub const TRIGGER_A: Selector<()> = Selector::new("monitor.update_status");
pub const TRIGGER_ON: Selector<()> = Selector::new("wrapper.focus_on");
pub const TRIGGER_OFF: Selector<()> = Selector::new("wrapper.focus_off");
pub const SCROLL_REQUEST: Selector<()> = Selector::new("wrapper.scroll");
//pub const TRIGGER_SYN: Selector<()> = Selector::new("wrapper.focus_syn");
pub const FINISH_SLOW_FUNCTION: Selector<Option<(usize, usize)>> =
    Selector::new("finish_slow_function");

#[derive(Default, Clone, Data, Lens)]
pub struct ApplicationState {
    pub error_message: Option<String>,
    pub current_book: Book,
    pub edit: bool,               // Serve a switchare da view mode a edit mode
    pub xml_backup: String,       // xml backup useful to discard changes done in edit mode
    pub modified: HashSet<usize>, //find better solution
    pub view: View,
    pub bookcase: BookCase,
    pub is_loading: bool,
    pub i_mode: bool, //use enum
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
            i_mode: false,
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

    pub fn get_current(&self) -> BookInfo {
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
            }
        }
    }
}
