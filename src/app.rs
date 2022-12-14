use crate::{Book, PageElement};
use druid::{im::Vector, Data, Lens, im::HashSet};
use crate::bookcase::{BookCase, BookInfo};

use crate::view::view::View;

#[derive(Default, Clone, Data, Lens)]
pub struct ApplicationState {
    pub current_book: Book,
    pub edit: bool, // Serve a switchare da view mode a edit mode
    pub xml_backup: String, // xml backup useful to discard changes done in edit mode
    pub modified: (bool, HashSet<usize>), //find better solution
    pub view: View,
    pub library: BookCase,
}

impl ApplicationState {
    pub fn new() -> ApplicationState {
        let mut app = ApplicationState {
            current_book: Book::empty_book(),
            edit: false,
            xml_backup: "".to_string(),
            modified: (false, HashSet::new()),
            view: View::new(),
            library: BookCase::new()
        };
        //app.update_view();
        app
    }

    pub fn update_view(&mut self){
        self.view.update_view(self.current_book.format_current_chapter());
    }

    pub fn get_library(&self) -> &Vector<BookInfo>{
        &(*self).library.library
    }

}


