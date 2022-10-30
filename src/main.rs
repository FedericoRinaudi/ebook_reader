mod book;

use std::path::PathBuf;
use druid::{AppLauncher, LocalizedString, Widget, WidgetExt, WindowDesc};
use druid::piet::InterpolationMode;
use druid::widget::{Button, CrossAxisAlignment, FillStrat, Flex, Image, LineBreaking, List, RawLabel, ViewSwitcher};

use crate::book::Book;
use crate::book::page_element::PageElement;


fn build_widget() -> impl Widget<Book> {
    let mut col = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
    let button_next = Button::new("next page").on_click(|_ctx, data: &mut Book, _env| {
        data.go_to_next_page_if_exist();
    });
    let button_prev = Button::new("prev page").on_click(|_ctx, data: &mut Book, _env| {
        data.go_to_prev_page_if_exist();
    });

    let mut row: Flex<Book> = Flex::row();
    row.add_child(button_prev);
    row.add_child(button_next);
    col.add_child(row.padding(30.0));

    let page = List::new(|| {
        ViewSwitcher::new(
            |data: &PageElement, _| data.clone() ,
            |_, data: &PageElement, _| -> Box<dyn Widget<PageElement>> {
                match data {
                    PageElement::Text(_) => {
                        let mut label = RawLabel::new();
                        label.set_line_break_mode(LineBreaking::WordWrap);
                        Box::new(label)
                    }
                    PageElement::Image(img_buf) => {
                        let mut img = Image::new(img_buf.clone());
                        img.set_fill_mode(FillStrat::ScaleDown);
                        img = img.interpolation_mode(InterpolationMode::Bilinear);
                        Box::new(img)
                    }
                }
            }
        )
    }).lens(Book::current_page);
    col.add_child(page.padding(30.0));
    col.scroll().vertical()
}


fn main() {

    //TODO: gestisco il caso in cui non sia possibile aprire l'ebook
    let initial_state = Book::new(PathBuf::from("./alices.epub"), 0, 0).unwrap();

    const WINDOW_TITLE :LocalizedString<Book> = LocalizedString::new("Hello World!");
    // describe the main window
    let main_window = WindowDesc::new(build_widget())
        .title(WINDOW_TITLE)
        .window_size((800.0, 1000.0));

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");

}