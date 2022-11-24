mod book;

use druid::widget::{Button, CrossAxisAlignment, FillStrat, Flex, FlexParams, Image, Label, LineBreaking, List, RawLabel, TextBox, ViewSwitcher};
use druid::{AppLauncher, Data, Lens, lens, LensExt, LocalizedString, Widget, WidgetExt, WindowDesc};
use std::path::PathBuf;

use crate::book::page_element::PageElement;
use crate::book::Book;


#[derive(Default, Clone, Data, Lens)]
pub struct ApplicationState {
    pub current_book: Book,
}
//SWITCH TRA VISUALIZZATORE ELENCO EBOOK E VISUALIZZATORE EBOOK
fn build_widget() -> impl Widget<ApplicationState> {
    let a: ViewSwitcher<ApplicationState, bool> = ViewSwitcher::new(
        |data: &ApplicationState, _| data.current_book.is_empty(),
        |_, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            if data.current_book.is_empty() {
                return Box::new(Button::new("libro").on_click(
                    |_ctx, data: &mut ApplicationState, _env| {
                        data.current_book = Book::new(PathBuf::from("./libro.epub"), 0, 0).unwrap();
                    },
                ));
            } else {
                  return Box::new(render_book())
            }
        },
    );
    a
}
//FUNZIONE CHE CREA I BOTTONI E FA VISUALIZZARE TESTO E IMMAGINI
fn render_book() -> impl Widget<ApplicationState> {
    let mut wrapper = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
    let mut row_due = Flex::row();
    let button_next = Button::new(">").on_click(|_ctx, data: &mut ApplicationState, _env| {
        data.current_book.go_to_next_page_if_exist();
    });
    let button_fast_forward =
        Button::new(">>").on_click(|_ctx, data: &mut ApplicationState, _env| {
            data.current_book.go_fast_forward_if_exist();
        });
    let button_prev = Button::new("<").on_click(|_ctx, data: &mut ApplicationState, _env| {
        data.current_book.go_to_prev_page_if_exist();
    });
    let button_fast_back = Button::new("<<").on_click(|_ctx, data: &mut ApplicationState, _env| {
        data.current_book.go_fast_back_if_exist();
    });
    let button_close_book =
        Button::new("close book").on_click(|_ctx, data: &mut ApplicationState, _env| {
            data.current_book = Book::empty_book();
        });

    let switch_mode = ViewSwitcher::new(
        |data: &ApplicationState, _| data.current_book.clone(),
        |_, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            let tag:&str;
            if data.current_book.edit {tag = "Read"} else {tag = "Edit"};
            let switch = Button::new(tag).on_click(|_ctx, data: &mut ApplicationState, _env| {
                if data.current_book.edit {data.current_book.save_n_update()};
                data.current_book.edit = !data.current_book.edit;
            });
            Box::new(switch)
        });

    let lbl_num_pag = Label::new(|data: &ApplicationState, _env: &_| {
        format!("{}", data.current_book.get_current_page_number())
    });

    let mut row: Flex<ApplicationState> = Flex::row();
    row.add_child(button_fast_back);
    row.add_child(button_prev);
    row.add_child(button_next);
    row.add_child(button_fast_forward);
    row.add_child(switch_mode);
    row_due.add_child(lbl_num_pag);

    row.add_child(button_close_book);
    //  col.add_child(row.padding(30.0));

    let page_with_scroll =
        ViewSwitcher::new(
            |data: &ApplicationState, _| data.current_book.edit,
            |_, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
                let mut col = Flex::column().cross_axis_alignment(CrossAxisAlignment::Baseline);
                if !data.current_book.edit {
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
                    col.add_child(page.padding(30.0).lens(ApplicationState::current_book));
                }else{

                    let xml_lens  = lens!(ApplicationState, current_book)
                        .then(lens!(Book, chapters_xml_and_path))
                        .index(data.current_book.current_chapter_number)
                        .then(lens!((String, String), 0));

                    let mut text = TextBox::new()
                        .with_line_wrapping(true)
                        .lens(xml_lens);

                    col.add_child(text);
                }
                Box::new(col.scroll().vertical())
            },
        );


    wrapper.add_child(Flex::row().fix_height(8.0));
    wrapper.add_flex_child(row, FlexParams::new(0.07, CrossAxisAlignment::Center));
    wrapper.add_child(Flex::row().fix_height(7.0));
    wrapper.add_flex_child(
        page_with_scroll.fix_width(700.0).fix_height(1000.0),
        FlexParams::new(0.92, CrossAxisAlignment::Baseline),
    );
    wrapper.add_child(Flex::row().fix_height(7.0));
    wrapper.add_flex_child(row_due, FlexParams::new(0.01, CrossAxisAlignment::Center));

    wrapper
}

fn main() {
    const WINDOW_TITLE: LocalizedString<ApplicationState> = LocalizedString::new("ebook reader");
    // describe the main window
    let main_window = WindowDesc::new(build_widget())
        .title(WINDOW_TITLE)
        .window_size((800.0, 1000.0));

    // start the application
    AppLauncher::with_window(main_window)
        .launch(ApplicationState {
            current_book: Book::empty_book()
        })
        .expect("Failed to launch application");
}
