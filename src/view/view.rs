use crate::{ApplicationState, PageElement};
use druid::{im::Vector, Data, Lens, LocalizedString};
use druid::piet::TextStorage;

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
    pub scroll_height: f64,
}
impl View {
    pub fn new() -> Self {
        View {
            window_size_view: VIEW_SIZE,
            window_size_edit: EDIT_SIZE,
            window_size_home: HOME_SIZE,
            current_view: Vector::new(),
            scroll_height: 0.0
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

    pub fn _set_window_size_home(&mut self, size: (f64, f64)) {
        (*self).window_size_home = size
    }

    pub fn get_view_size(&self, width:f64) -> usize {
        let mut size = 0;
        for text in self.current_view.iter() {
            match text {
                PageElement::Text(rt) => {
                    /* TODO: Following
                    The idea is to somehow produce a richtext width and confront it with the view width passed as parameter

                    possible library with discussion and example to use: https://github.com/fschutt/printpdf/issues/49
                     */

                    /*
                    Right now we just assume 100 chars each line to get an estimate of how many lines in the chapter
                    then we multiply the number of lines for the size of the chars which rn is 16
                    We are not considering images or fonttypes
                    Richtext doesnt have a way to get its attributes to dynamically modify the constant so another structure
                    should be used here or maybe the size could be calculated in the chapter.format() function and associated
                    with the richtext in the PageElement


                     */

                    let current_size = rt.as_str().chars().count()/100 +1; //TODO: make 100 a window_size based value
                    size += current_size*16


                },
                _ => ()
            }
        }
        size
    }


}
