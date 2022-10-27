mod book;

use std::path::PathBuf;
use druid::{Widget, LocalizedString, WindowDesc, AppLauncher, WidgetExt};
use druid::widget::{Flex, Button, CrossAxisAlignment, List, RawLabel, LineBreaking};

use crate::book::Book;


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
        let mut label = RawLabel::new();
        label.set_line_break_mode(LineBreaking::WordWrap);
        label
    }).lens(Book::current_page);
    col.add_child(page.padding(30.0));
    col.scroll().vertical()
}


fn main() {

    //TODO: gestisco il caso in cui non sia possibile aprire l'ebook
    let initial_state = Book::new(PathBuf::from("./libro.epub")).unwrap();

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