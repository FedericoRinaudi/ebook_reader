use crate::{Book, PageElement};
use druid::{im::Vector, Data, Lens};

#[derive(Default, Clone, Data, Lens)]
pub struct ApplicationState {
    pub current_book: Book,
    pub edit: bool, // Serve a switchare da view mode a edit mode
    pub window_size: (f64, f64), //TODO: Crea Struct di riferimento per size o usa l'env
    pub current_view: Vector<PageElement>
    // library: Vector<BookInfo>,
}

impl ApplicationState {
    pub fn update_view(&mut self){
        self.current_view = self.current_book.format_current_chapter()
    }
}