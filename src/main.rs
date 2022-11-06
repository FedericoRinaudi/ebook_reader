mod book;

use druid::widget::{
    Button, CrossAxisAlignment, FillStrat, Flex, Image, LineBreaking, List, RawLabel, ViewSwitcher,
};
use druid::{AppLauncher, LocalizedString, Widget, WidgetExt, WindowDesc, Data, Lens};
use std::path::PathBuf;

use crate::book::page_element::PageElement;
use crate::book::Book;

#[derive(Default, Clone, Data, Lens)]
pub struct ApplicationState{
    pub current_book: Book
}

fn build_widget() -> impl Widget<ApplicationState> {
    let a: ViewSwitcher<ApplicationState, bool> = ViewSwitcher::new(
        |data: &ApplicationState, _|data.current_book.is_empty(),
        |_, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            if data.current_book.is_empty() {
                return Box::new(Button::new("libro").on_click(|_ctx, data: &mut ApplicationState, _env| {
                    data.current_book = Book::new(PathBuf::from("./libro.epub"), 0, 0).unwrap();
                }));
            } else {
                return Box::new(render_book());
            }
        });
    a
}

fn render_book() -> impl Widget<ApplicationState> {
    let mut col = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
    let button_next = Button::new("next page").on_click(|_ctx, data: &mut ApplicationState, _env| {
        data.current_book.go_to_next_page_if_exist();
    });
    let button_prev = Button::new("prev page").on_click(|_ctx, data: &mut ApplicationState, _env| {
        data.current_book.go_to_prev_page_if_exist();
    });
    let button_close_book = Button::new("close book").on_click(|_ctx, data: &mut ApplicationState, _env| {
        data.current_book = Book::empty_book();
    });

    let mut row: Flex<ApplicationState> = Flex::row();
    row.add_child(button_prev);
    row.add_child(button_next);
    row.add_child(button_close_book);
    col.add_child(row.padding(30.0));

    let page =
        List::new(|| {
            ViewSwitcher::new(
                |data: &PageElement, _| data.clone(),
                |_, data: &PageElement, _| -> Box<dyn Widget<PageElement>> {
                    match data {
                        PageElement::Text(_) => {
                            let mut label = RawLabel::new();
                            label.set_line_break_mode(LineBreaking::WordWrap);
                            Box::new(label)
                        }
                        PageElement::Image(img_buf) => Box::new(Flex::row().with_child(
                            Image::new(img_buf.clone()).fill_mode(FillStrat::ScaleDown),
                        )),
                    }
                },
            )
        }).lens(Book::current_page);
        //.lens(ApplicationState::current_book::current_page);
    col.add_child(page.padding(30.0).lens(ApplicationState::current_book));
    col.scroll().vertical()
}

fn main() {
    //TODO: gestisco il caso in cui non sia possibile aprire l'ebook
    //let initial_state = Book::new(PathBuf::from("./libro.epub"), 0, 0).unwrap();

    const WINDOW_TITLE: LocalizedString<ApplicationState> = LocalizedString::new("ebook reader");
    // describe the main window
    let main_window = WindowDesc::new(build_widget())
        .title(WINDOW_TITLE)
        .window_size((800.0, 1000.0));

    // start the application
    AppLauncher::with_window(main_window)
        .launch(ApplicationState{current_book: Book::empty_book()})
        .expect("Failed to launch application");
}
