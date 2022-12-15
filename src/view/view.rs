use crate::{ApplicationState, PageElement};
use druid::{im::Vector, Data, Lens, LocalizedString};

pub const WINDOW_TITLE: LocalizedString<ApplicationState> =
    LocalizedString::new("Ebook Reader - Library");
const VIEW_SIZE: (f64, f64) = (800.0, 1000.0);
const EDIT_SIZE: (f64, f64) = (1600.0, 1000.0);
const HOME_SIZE: (f64, f64) = (800.0, 1000.0);

#[derive(Default, Clone, Data, Lens)]
pub struct View {
    window_size_view: (f64, f64),
    window_size_edit: (f64, f64),
    window_size_home: (f64, f64),
    pub current_view: Vector<PageElement>,
}
impl View {
    pub fn new() -> Self {
        View {
            window_size_view: VIEW_SIZE,
            window_size_edit: EDIT_SIZE,
            window_size_home: HOME_SIZE,
            current_view: Vector::new(),
        }
    }

    pub fn update_view(&mut self, vec: Vector<PageElement>) {
        self.current_view = vec
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

    pub fn set_window_size_home(&mut self, size: (f64, f64)) {
        (*self).window_size_home = size
    }
}
