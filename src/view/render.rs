use crate::book::{chapter::Chapter, Book};
use crate::bookcase::{BookCase, BookInfo};
use crate::controllers::Update;
use crate::view::buttons::Buttons;
use crate::view::view::View;
use crate::{ApplicationState, PageElement};
use druid::widget::{
    Button, Click, ControllerHost, CrossAxisAlignment, FillStrat, Flex, FlexParams, Image,
    LineBreaking, List, RawLabel, TextBox, ViewSwitcher,
};
use druid::{lens, ImageBuf, LensExt, Widget, WidgetExt};
use std::path::PathBuf;

//SWITCH TRA VISUALIZZATORE ELENCO EBOOK E VISUALIZZATORE EBOOK
pub fn build_main_view() -> impl Widget<ApplicationState> {
    let main_nav: ViewSwitcher<ApplicationState, bool> = ViewSwitcher::new(
        |data: &ApplicationState, _| data.current_book.is_empty(), /* Condizione della useEffect (?) */
        |_ctx, data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>> {
            if data.current_book.is_empty() {
                /* Renderizziamo la libreria di libri disponibili */
                Box::new(render_library())
                //Box::new(render_book())
            } else {
                /* Renderizziamo il libro scelto */
                Box::new(render_book())
            }
        },
    );
    main_nav
}

//FUNZIONE CHE CREA I BOTTONI E FA VISUALIZZARE TESTO E IMMAGINI
fn render_book() -> impl Widget<ApplicationState> {
    let mut wrapper = Flex::column(); //.cross_axis_alignment(CrossAxisAlignment::Start);

    /*let button_close_book =
    Button::new("close book").on_click(|_ctx, data: &mut ApplicationState, _env| {
        let mut output = OpenOptions::new()
            .append(true)
            .create(true)
            .open("./tmp.txt")
            .expect("Unable to open file");
        let input = BufReader::new(File::open("file.txt").expect("Cannot open file.txt"));
        for line in input.lines() {
            if !(line
                .as_ref()
                .unwrap()
                .clone()
                .split_whitespace()
                .next()
                .unwrap()
                .to_string()
                == data.current_book.get_path())
            {
                output
                    .write_all((line.unwrap().clone() + "\n").as_bytes())
                    .expect("TODO: panic message");
            } else {
                let _ = output.write_all(
                    (data.current_book.get_path()
                        + " "
                        + data
                            .current_book
                            .get_current_chapter_number()
                            .to_string()
                            .as_str()
                        + " "
                        + data
                            .current_book
                            .get_current_page_number_in_chapter()
                            .to_string()
                            .as_str()
                        + " "
                        + data
                            .current_book
                            .get_current_page_number()
                            .to_string()
                            .as_str()
                        + " "
                        + data
                            .current_book
                            .get_image(data.current_book.get_path())
                            .to_string()
                            .as_str()
                        + "\n")
                        .as_bytes(),
                );
            }
        }
        let _ = fs::remove_file("file.txt");
        let _ = fs::rename("tmp.txt", "file.txt");

        data.library = read_from_file();
        data.current_book = Book::empty_book();
    });*/

    /* Switcha la modalità dell'app */
    let buttons = ViewSwitcher::new(
        |data: &ApplicationState, _| data.edit,
        move |_, data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>> {
            if data.edit {
                Box::new(ViewSwitcher::new(
                    |data: &ApplicationState, _| {
                        data.xml_backup
                            == data.current_book.chapters[data.current_book.get_nav().get_ch()].xml
                    },
                    |cond, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
                        let mut row: Flex<ApplicationState> = Flex::row();
                        if *cond {
                            row.add_child(Buttons::bnt_view());
                            row.add_child(Buttons::btn_save());
                        } else {
                            row.add_child(Buttons::btn_confirm());
                            row.add_child(Buttons::bnt_discard());
                        }
                        Box::new(row)
                    },
                ))
            } else {
                let mut row: Flex<ApplicationState> = Flex::row();
                row.add_child(Buttons::btn_prev());
                row.add_child(Buttons::btn_edit());
                row.add_child(Buttons::btn_save());
                row.add_child(Buttons::btn_next());
                Box::new(row)
            }
        },
    );

    // row.add_child(button_close_book);
    // col.add_child(row.padding(30.0));

    let scrollable_text = ViewSwitcher::new(
        |data: &ApplicationState, _| data.edit,
        |_, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            if !data.edit {
                /* VIEW MODE */
                Box::new(render_view_mode().padding(30.0).scroll().vertical())
            } else {
                /* EDIT MODE */
                Box::new(render_edit_mode().padding(0.0))
            }
        },
    );

    wrapper.add_child(Flex::row().fix_height(7.0));
    wrapper.add_flex_child(buttons, FlexParams::new(0.07, CrossAxisAlignment::Center));
    wrapper.add_child(Flex::row().fix_height(7.0));

    wrapper.add_flex_child(scrollable_text.fix_height(1000.0), 0.9);
    wrapper.add_child(Flex::row().fix_height(1.0));

    wrapper
}

fn render_edit_mode() -> Flex<ApplicationState> {
    let mut viewport = Flex::row();
    let view = ViewSwitcher::new(
        |data: &ApplicationState, _env| {
            data.current_book.chapters[data.current_book.get_nav().get_ch()]
                .xml
                .clone()
        },
        |_vec, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            Box::new(render_view_mode())
        },
    )
    .scroll()
    .vertical();
    let edit = ViewSwitcher::new(
        |data: &ApplicationState, _| data.current_book.get_nav().get_ch(),
        |_, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            let xml_lens = lens!(ApplicationState, current_book)
                .then(lens!(Book, chapters))
                .index(data.current_book.get_nav().get_ch())
                .then(lens!(Chapter, xml));

            let editable_xml = TextBox::new().with_line_wrapping(true).lens(xml_lens);

            /* Permette di modificare in xml l'appstate*/
            let host = ControllerHost::new(
                editable_xml,
                Update::new(|_, data: &mut ApplicationState, _| data.update_view()),
            );

            let mut xml = Flex::column().cross_axis_alignment(CrossAxisAlignment::Baseline);
            xml.add_child(host);
            Box::new(xml)
        },
    );

    viewport.add_flex_child(edit.padding(10.0).scroll().vertical(), 0.5);
    viewport.add_flex_child(view.padding(10.0), 0.5);
    viewport
}

fn render_view_mode() -> Flex<ApplicationState> {
    let mut viewport = Flex::column().cross_axis_alignment(CrossAxisAlignment::Baseline);
    /*
    let lens = lens!(ApplicationState, current_book)
        .map(|book| book.format_current_chapter(), |_ , _|());
        */
    let lens = lens!(ApplicationState, view).then(lens!(View, current_view));

    let chapter =
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
                        PageElement::Error(_e) => {
                            let mut label = RawLabel::new();
                            label.set_line_break_mode(LineBreaking::WordWrap);
                            Box::new(label)
                        }
                    }
                },
            )
        })
        .lens(lens);
    viewport.add_child(chapter);
    viewport
}

fn render_library() -> Flex<ApplicationState> {
    /*
    let mut col = Flex::column();
    let mut row = Flex::row();
    //let lib = data.library.clone();
    let row_flex = 1.0 / ((data.get_library().len() as f64 / 3.0) + 1.0);
    for (index, book_info) in data.get_library().iter().enumerate() {

        /*  FORSE INUTILE
            let b = Button::new(e.name.clone()).on_click(move |_ctx, button_data: &mut ApplicationState, _env| {
            button_data.current_book = Book::new(PathBuf::from(e.name.clone()), e.start_chapter, e.start_page_in_chapter, e.tot_pages).unwrap();
        });
        */

        let clickable_image = ControllerHost::new(
            Image::new(ImageBuf::from_file(book_info.cover_path.clone())
                .unwrap())//TODO: unwrap_or(default image)
                .fix_width(300.0)
                .fix_height(200.0),
            Click::new(move |_ctx, data: &mut ApplicationState, _env| {
                data.current_book = Book::new(
                    PathBuf::from(book_info.name.clone()),
                    book_info.start_chapter,
                    None
                ).unwrap();
                data.update_view();
            }),
        );
        row.add_flex_child(clickable_image, FlexParams::new(row_flex, CrossAxisAlignment::Start));
        if index != 0 && (index + 1) % 3 == 0 {
            col.add_flex_child(row, FlexParams::new(0.3, CrossAxisAlignment::Center));
            row = Flex::row();
        }
    }
    col.add_child(row);
    col.scroll().vertical()
        */

    let mut viewport = Flex::column().cross_axis_alignment(CrossAxisAlignment::Baseline);
    /*
    let lens = lens!(ApplicationState, current_book)
        .map(|book| book.format_current_chapter(), |_ , _|());
        */
    let lens = lens!(ApplicationState, bookcase).then(lens!(BookCase, library));

    let home_page = List::new(|| {
        ViewSwitcher::new(
            |data: &BookInfo, _| data.clone(),
            |info, data: &BookInfo, _| -> Box<dyn Widget<BookInfo>> {
                Box::new(
                    ControllerHost::new(Image::new(ImageBuf::from_file(info.cover_path.clone()).unwrap()) //TODO: unwrap_or(default image)
                        .fix_width(300.0)
                        .fix_height(200.0),
                    Click::new(move |_ctx, data: &mut ApplicationState, _env| {
                        data.current_book =
                            Book::new(PathBuf::from(info.name.clone()), info.start_chapter, None).unwrap();
                        data.update_view();
                    }),
                ))
            },
        )
    })
    .lens(lens);
    viewport.add_child(home_page);
    viewport
}
