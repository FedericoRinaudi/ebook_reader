use crate::{Book, PageElement};
use druid::{im::Vector, Data, Lens, im::HashSet};

use crate::view::view::View;

#[derive(Default, Clone, Data, Lens)]
pub struct ApplicationState {
    pub current_book: Book,
    pub edit: bool, // Serve a switchare da view mode a edit mode
    pub xml_backup: String, // xml backup useful to discard changes done in edit mode
    pub modified: HashSet<usize>,
    pub view: View
    // library: Vector<BookInfo>,
}

impl ApplicationState {
    pub fn new(current_book: Book) -> ApplicationState {
        let mut app = ApplicationState {
            current_book,
            edit: false,
            xml_backup: "".to_string(),
            modified: HashSet::new(),
            view: View::new()
        };
        app.update_view();
        app
    }

    pub fn update_view(&mut self){
        self.view.update_view(self.current_book.format_current_chapter());
    }

}


