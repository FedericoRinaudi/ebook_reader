use crate::book::page_element::PageElement;
use crate::book::{chapter::Chapter, Book};
use crate::bookcase::{BookCase, BookInfo};
use crate::controllers::Update;
use crate::formatters::CustomFormatter;
use crate::ocr::{Mapping, OcrData};
use crate::view::buttons::Buttons;
use crate::view::view::View;
use crate::widgets::custom_img::BetterImage;
use crate::widgets::custom_label::BetterLabel;
use crate::widgets::custom_scrolls::{BetterScroll, SyncScroll};
use crate::widgets::custom_tooltip::TipExt;
use crate::{ApplicationState, ContentType};
use druid::widget::{
    Container, ControllerHost, CrossAxisAlignment, Flex, FlexParams, Image, Label, LineBreaking,
    List, Padding, Painter, RawLabel, Scroll, Spinner, TextBox, ViewSwitcher,
};
use druid::{lens, Color, Env, LensExt, RenderContext, Widget, WidgetExt};

//SWITCH TRA VISUALIZZATORE ELENCO EBOOK E VISUALIZZATORE EBOOK
pub fn build_main_view() -> impl Widget<ApplicationState> {
    Flex::column()
        .with_child(ViewSwitcher::new(
            |data: &ApplicationState, _| data.error_message.clone(), /* Ad ora non funziona... lo fixo */
            |msg, _data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>> {
                let mut error_row = Flex::row().must_fill_main_axis(true);
                match msg {
                    None => {}
                    Some(msg) => {
                        error_row.add_child(Padding::new(10.0, Label::new(msg.to_string()).with_text_color(Color::rgb(0.9, 0.05, 0.05)).with_text_size(14.)));
                        error_row.add_flex_spacer(1.0);
                        error_row.add_child(Padding::new(10.0, Buttons::btn_close_error()));
                    }
                }
                Box::new(error_row.background(Color::rgb(0.247, 0.194, 0.182)))
            },
        )
        )
        .with_flex_child(ViewSwitcher::new(
            |data: &ApplicationState, _| data.is_loading, /* Ad ora non funziona... lo fixo */
            |_load, data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>> {
                if !data.is_loading {
                    Box::new(
                        ViewSwitcher::new(
                            |data: &ApplicationState, _| data.book_to_align.is_empty(),
                            |cond, _data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>> {
                                return if *cond {
                                    Box::new(ViewSwitcher::new(
                                        |data: &ApplicationState, _| data.book_to_view.is_empty(), /* Condizione della useEffect (?) */
                                        |_, data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>> {
                                            if data.book_to_view.is_empty() {
                                                /* Renderizziamo la libreria di libri disponibili */
                                                Box::new(render_library())
                                                //Box::new(render_book())
                                            } else {
                                                /* Renderizziamo il libro scelto */
                                                Box::new(Padding::new((0.0, 0.0, 0.0, 18.0), render_book()))
                                            }
                                        },
                                    ))
                                } else {
                                    Box::new(render_ocr_syn())
                                }
                            }))
                } else {
                    Box::new(Spinner::new().fix_height(40.0).center())
                }
            },
        ), 1.)
}

//FUNZIONE CHE CREA I BOTTONI E FA VISUALIZZARE TESTO E IMMAGINI
fn render_book() -> impl Widget<ApplicationState> {
    /* Switcha la modalità dell'app */
    ViewSwitcher::new(
        |data: &ApplicationState, _| data.edit,
        move |_, data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>> {
            if data.edit {
                let mut window = Flex::column();
                let buttons = ViewSwitcher::new(
                    |data: &ApplicationState, _| {
                        data.xml_backup
                            == data.book_to_view.chapters[data.book_to_view.get_nav().get_ch()].xml
                    },
                    |cond, _data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
                        Box::new(if *cond {
                            Flex::row()
                                .with_child(Buttons::btn_view())
                                .with_spacer(20.0)
                                .with_child(Buttons::btn_save())
                        } else {
                            Flex::row()
                                .with_child(Buttons::btn_discard())
                                .with_spacer(20.0)
                                .with_child(Buttons::btn_confirm())
                        })
                    },
                );
                let screen = render_edit_mode();

                window.add_child(Flex::row().fix_height(7.0));
                window.add_flex_child(buttons, FlexParams::new(0.07, CrossAxisAlignment::Center));
                window.add_child(Flex::row().fix_height(7.0));
                window.add_flex_child(screen, 0.9);
                window.add_child(Flex::row().fix_height(1.0));
                Box::new(window)
            } else {
                let mut window = Flex::column();
                let buttons: Flex<ApplicationState> = Flex::row()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_flex_child(Buttons::btn_prev(), 0.1)
                    .with_flex_spacer(0.3)
                    .with_flex_child(Buttons::btn_edit(), 0.1)
                    .with_flex_child(Buttons::btn_save(), 0.1)
                    .with_flex_child(Buttons::btn_close_book(), 0.1)
                    .with_flex_spacer(0.3)
                    .with_flex_child(Buttons::btn_next(), 0.1);
                let screen = BetterScroll::new(render_view_mode());

                window.add_child(Flex::row().fix_height(7.0));
                window.add_flex_child(buttons, FlexParams::new(0.07, CrossAxisAlignment::Center));
                window.add_child(Flex::row().fix_height(7.0));
                window.add_flex_child(Padding::new((0.0, 0.0, 0.0, 7.0), screen), 0.9);
                Box::new(window)
            }
        },
    )
}

fn render_edit_mode() -> impl Widget<ApplicationState> {
    let mut viewport = Flex::row();
    let view = Scroll::new(render_view_mode()).vertical();
    let edit = ViewSwitcher::new(
        |data: &ApplicationState, _| data.book_to_view.get_nav().get_ch(),
        |_, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            let xml_lens = lens!(ApplicationState, book_to_view)
                .then(lens!(Book, chapters))
                .index(data.book_to_view.get_nav().get_ch())
                .then(lens!(Chapter, xml));

            let editable_xml = TextBox::new().with_line_wrapping(true).lens(xml_lens);

            /* Permette di modificare in xml l'appstate*/
            let host = ControllerHost::new(
                editable_xml,
                Update::new(|ctx, data: &mut ApplicationState, _| {
                    data.update_view(ctx.get_external_handle())
                }),
            );

            let xml = Scroll::new(host).vertical();
            Box::new(xml)
        },
    );

    viewport.add_flex_child(SyncScroll::new(edit.padding(10.0)), 0.5);
    viewport.add_flex_child(view.padding(10.0), 0.5);
    viewport.padding(0.0)
}

fn render_view_mode() -> impl Widget<ApplicationState> {
    ViewSwitcher::new(
        |data: &ApplicationState, _| data.view.current_view.clone(),
        move |_, _data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            let mut viewport = Flex::column().cross_axis_alignment(CrossAxisAlignment::Baseline);

            let lens = lens!(ApplicationState, view).then(lens!(View, current_view));

            let chapter = List::new(|| {
                ViewSwitcher::new(
                    |data: &PageElement, _| data.content.clone(),
                    |ele, data: &PageElement, _| -> Box<dyn Widget<PageElement>> {
                        match &ele {
                            ContentType::Text(_) => {
                                return if data.pg_offset.0 != 0 {
                                    Box::new(BetterLabel::new().tooltip(
                                        |data: &PageElement, _env: &Env| {
                                            let mut po = "Page ".to_string();
                                            if (*data).pg_offset.1 == true {
                                                po.push_str(&((*data).pg_offset.0 - 1).to_string());
                                                po.push_str("-");
                                            };
                                            po.push_str(&data.pg_offset.0.to_string());
                                            String::from(po)
                                        },
                                        true,
                                    ))
                                } else {
                                    Box::new(BetterLabel::new())
                                }
                            }
                            ContentType::Image(img_buf) => {
                                if data.pg_offset.0 != 0 {
                                    Box::new(BetterImage::new(img_buf.clone()).tooltip(
                                        |data: &PageElement, _env: &Env| {
                                            let mut str = String::from(
                                                "Page ".to_owned() + &data.pg_offset.0.to_string(),
                                            );
                                            let str2 = String::from(
                                                "-".to_owned()
                                                    + &(data.pg_offset.0 - 1).to_string(),
                                            );
                                            str.push_str(if data.pg_offset.1 { &str2 } else { "" });
                                            str
                                        },
                                        true,
                                    ))
                                } else {
                                    Box::new(BetterImage::new(img_buf.clone()))
                                }
                            }
                            ContentType::Error(_e) => {
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
            Box::new(Padding::new((30.0, 0.0, 30.0, 0.0), viewport))
        },
    )
}

fn render_library() -> impl Widget<ApplicationState> {
    ViewSwitcher::new(
        |data: &ApplicationState, _| data.bookcase.library.clone(),
        |_app, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            // TODO:Load IMAGES IN THREAD
            let mut col = Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .with_spacer(12.0)
                .with_child(
                    Flex::row()
                        .with_child(
                            Label::new(String::from("Your Library"))
                                .with_text_size(40.0)
                                .padding(30.0),
                        )
                        .with_flex_spacer(0.7)
                        .with_child(Padding::new(20., Buttons::btn_add_book())),
                );
            for (i, book_info) in data.get_library().clone().into_iter().enumerate() {
                let mut pill = Flex::row()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .must_fill_main_axis(true);

                let uno = Flex::column()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_child(
                        Image::new(book_info.cover_buf.clone())
                            .fix_width(300.0)
                            .fix_height(200.0),
                    );

                let due = Flex::column()
                    .with_child(
                        Flex::row()
                            .must_fill_main_axis(true)
                            .cross_axis_alignment(CrossAxisAlignment::Start)
                            .with_child(Buttons::btn_read_book(book_info.clone()))
                            .with_spacer(10.0)
                            .with_child(Buttons::btn_remove_book(i))
                            .with_spacer(10.0)
                            .with_child(Buttons::btn_ocr(book_info.clone()))
                            .with_spacer(10.0)
                            .with_child(Buttons::btn_ocr_syn(i)), //HERE
                    )
                    .with_spacer(15.0)
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_child(
                        Label::new(&*book_info.title.clone())
                            .with_text_size(25.0)
                            .with_line_break_mode(LineBreaking::WordWrap),
                    )
                    .with_spacer(4.0)
                    .with_child(print_card_element("By", &book_info.creator))
                    .with_spacer(3.0)
                    .with_child(print_card_element("Language", &book_info.language))
                    .with_spacer(3.0)
                    .with_child(print_card_element("Directory", &book_info.path));

                /*.with_child(Label::new(
                    String::from("Chapter: ") + &*book_info.start_chapter.clone().to_string(),
                ))*/
                /*.with_child(Label::new(
                    String::from("Offset: ")
                        + &*book_info.start_element_number.clone().to_string(),
                ))*/

                pill.add_flex_child(Padding::new((0.0, 2.0, 10.0, 2.0), uno), 0.2);
                pill.add_flex_child(Padding::new((0.0, 0.0, 0.0, 10.0), due), 0.8);

                /*let wrap = Container::new(pill)
                .border(Color::WHITE, 1.0)
                .rounded(8.0);*/

                col.add_child(Padding::new((12.0, 0.0, 12.0, 8.0), pill));
                if i != (data.get_library().len() - 1) {
                    col.add_child(
                        Painter::new(|ctx, _: &_, _: &_| {
                            let size = ctx.size().to_rect();
                            ctx.fill(size, &Color::WHITE)
                        })
                        .fix_height(1.0)
                        .padding(20.0),
                    );
                }
            }
            Box::new(col.scroll().vertical())
        },
    )
}

fn print_card_element(label: &str, value: &str) -> impl Widget<ApplicationState> {
    return if value != "" {
        Flex::row()
            .with_flex_child(Label::new(String::from(label.to_owned() + ":   ")), 1.)
            .with_flex_child(
                Label::new(value.to_string())
                    .with_text_color(Color::grey(0.5))
                    .with_line_break_mode(LineBreaking::WordWrap),
                4.,
            )
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .must_fill_main_axis(true)
    } else {
        Flex::row()
    };
}


fn render_ocr_syn() -> impl Widget<ApplicationState> {
    ViewSwitcher::new(
        |data: &ApplicationState, _| data.view.ocr_form_stage,
        |stage, data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>> {
            match stage {
                1 => {
                    Box::new(
                        Flex::column()
                            .with_child(
                                Flex::row()
                                    .with_flex_spacer(1.)
                                    .with_child(
                                        Label::new("1").with_text_size(25.))
                                    .with_child(
                                        Label::new("2").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("3").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("4").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("5").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("6").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_flex_spacer(1.)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Label::new("Thanks to this function while reading the selected epub you can know the corresponding page of the paper book at any time.\
                                \nIn order to be able to do this you will need to upload photos of a couple of pages of the paper book and you will have to a verify the correctness of some information from the photos.\
                                \nPress 'NEXT' to proceed or 'LIBRARY' to return to the home page.")
                                    .with_text_size(18.)
                                    .with_text_color(Color::grey(0.9))
                                    .with_line_break_mode(LineBreaking::WordWrap)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Flex::row()
                                    .must_fill_main_axis(true)
                                    .with_flex_spacer(1.)
                                    .with_child(Buttons::btn_ocr_form_close(true))
                                    .with_spacer(5.)
                                    .with_child(Buttons::btn_ocr_form_next())
                                    .with_flex_spacer(1.)
                            )
                            .padding(20.)
                    )
                }
                2 => {
                    Box::new(
                        Flex::column()
                        .with_child(
                            Flex::row()
                                .with_flex_spacer(1.)
                                .with_child(
                                    Label::new("1").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                .with_child(
                                    Label::new("2").with_text_size(25.))
                                .with_child(
                                    Label::new("3").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                .with_child(
                                    Label::new("4").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                .with_child(
                                    Label::new("5").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                .with_child(
                                    Label::new("6").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                .with_flex_spacer(1.)
                        )
                        .with_spacer(20.)
                        .with_child(
                            Label::new("Here you need to upload a picture of the first page of the first chapter.\
                                                \nFrom this page will start the alignment being the first for which there is a match between ebook and paper book.\
                                                \nPress 'LOAD PAGE' to load the page, 'GO BACK' to return to '2' or 'LIBRARY' to return to the home page.")
                                .with_text_size(18.)
                                .with_text_color(Color::grey(0.9))
                                .with_line_break_mode(LineBreaking::WordWrap)
                        )
                        .with_spacer(20.)
                        .with_child(
                            Flex::row()
                                .must_fill_main_axis(true)
                                .with_flex_spacer(1.)
                                .with_child(Buttons::btn_ocr_form_close(true))
                                .with_spacer(5.)
                                .with_child(Buttons::btn_ocr_form_prev())
                                .with_spacer(5.)
                                .with_child(Buttons::btn_add_first_page())
                                .with_flex_spacer(1.)
                        )
                        .padding(20.))
                }
                3 => {
                    Box::new(
                        Flex::column()
                            .with_child(
                                Flex::row()
                                    .with_flex_spacer(1.)
                                    .with_child(
                                        Label::new("1").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("2").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("3").with_text_size(25.))
                                    .with_child(
                                        Label::new("4").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("5").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("6").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_flex_spacer(1.)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Label::new("Perfect! We recognized the page you uploaded.\
                                                      \nNow you would need to check if the number of total rows on the page (counting also the title, but not the page number and any headers) and the page number we calculated are correct, if not, correct them.\
                                                      \nPress 'CONFIRM' to confirm, 'GO BACK' to return to '2' or 'LIBRARY' to return to the home page.")
                                    .with_text_size(18.)
                                    .with_text_color(Color::grey(0.9))
                                    .with_line_break_mode(LineBreaking::WordWrap)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Flex::row()
                                    .with_flex_spacer(1.)
                                    .with_child(Label::new(String::from("PAGE:")).with_text_color(Color::grey(0.9)))
                                    .with_spacer(5.)
                                    .with_child(ocr_form(data.get_current_book_info().ocr.first.unwrap(), data, OcrFormFields::PageNum))
                                    .with_spacer(20.)
                                    .with_child(Label::new(String::from("LINES:")).with_text_color(Color::grey(0.9)))
                                    .with_spacer(5.)
                                    .with_child(ocr_form(data.get_current_book_info().ocr.first.unwrap(), data, OcrFormFields::NumLines))
                                    .with_flex_spacer(1.)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Flex::row()
                                    .must_fill_main_axis(true)
                                    .with_flex_spacer(1.)
                                    .with_child(Buttons::btn_ocr_form_close(true))
                                    .with_spacer(5.)
                                    .with_child(Buttons::btn_submit_ocr_form0())
                                    .with_spacer(5.)
                                    .with_child(Buttons::btn_remove_first_page())
                                    .with_flex_spacer(1.)
                            )
                            .padding(20.)
                    )
                }
                4 =>{
                    Box::new(
                        Flex::column()
                            .with_child(
                                Flex::row()
                                    .with_flex_spacer(1.)
                                    .with_child(
                                        Label::new("1").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("2").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("3").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("4").with_text_size(25.))
                                    .with_child(
                                        Label::new("5").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("6").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_flex_spacer(1.)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Label::new("Here you need to upload an image of a page from the book.\
                                                      \nYou need to upload a picture of a page with little dialogue (possibly long periods).\
                                                      \nThe page must not be truncated for some reason.\
                                                      \nUsually any page taken in the middle of a chapter will meet the requirements.\
                                                      \nPress 'LOAD PAGE' to load the page, 'GO BACK' to return to '3' or 'LIBRARY' to return to the home page.")
                                    .with_text_size(18.)
                                    .with_text_color(Color::grey(0.9))
                                    .with_line_break_mode(LineBreaking::WordWrap)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Flex::row()
                                    .must_fill_main_axis(true)
                                    .with_flex_spacer(1.)
                                    .with_child(Buttons::btn_ocr_form_close(true))
                                    .with_spacer(5.)
                                    .with_child(Buttons::btn_ocr_form_prev())
                                    .with_spacer(5.)
                                    .with_child(Buttons::btn_add_other_page())
                                    .with_flex_spacer(1.)
                            )
                            .padding(20.))
                }
                5 =>{
                    Box::new(
                        Flex::column()
                            .with_child(
                                Flex::row()
                                    .with_flex_spacer(1.)
                                    .with_child(
                                        Label::new("1").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("2").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("3").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("4").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("5").with_text_size(25.))
                                    .with_child(
                                        Label::new("6").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_flex_spacer(1.)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Label::new("Perfect! We recognized the page you uploaded.\
                                                      \nNow you would need to check if the number of total rows on the page (counting also the title, but not the page number and any headers) we calculated is correct, if not, correct them.\
                                                      \nPress 'CONFIRM' to confirm, 'GO BACK' to return to '4' or 'LIBRARY' to return to the home page.")
                                    .with_text_size(18.)
                                    .with_text_color(Color::grey(0.9))
                                    .with_line_break_mode(LineBreaking::WordWrap)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Flex::row()
                                    .with_flex_spacer(1.)
                                    .with_child(Label::new(String::from("LINES:")).with_text_color(Color::grey(0.9)))
                                    .with_spacer(5.)
                                    .with_child(ocr_form(data.get_current_book_info().ocr.other.unwrap(), data, OcrFormFields::NumLines))
                                    .with_flex_spacer(1.)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Flex::row()
                                    .must_fill_main_axis(true)
                                    .with_flex_spacer(1.)
                                    .with_child(Buttons::btn_ocr_form_close(true))
                                    .with_spacer(5.)
                                    .with_child(Buttons::btn_submit_ocr_form1())
                                    .with_spacer(5.)
                                    .with_child(Buttons::btn_remove_other_page())
                                    .with_flex_spacer(1.)
                            )
                            .padding(20.)
                    )
                }
                6 =>{
                    Box::new(
                        Flex::column()
                            .with_child(
                                Flex::row()
                                    .with_flex_spacer(1.)
                                    .with_child(
                                        Label::new("1").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("2").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("3").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("4").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("5").with_text_size(25.).with_text_color(Color::grey(0.5)))
                                    .with_child(
                                        Label::new("6").with_text_size(25.))
                                    .with_flex_spacer(1.)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Label::new("Perfect!\
                                                      \nEverything went well and now your book is aligned with the paper version.\
                                                      \nNow clicking on a paragraph (only if it is after the first page you uploaded) will show you the page number on the physical book!\
                                                      \nPress 'LIBRARY' to return to the home page. ")
                                    .with_text_size(18.)
                                    .with_text_color(Color::grey(0.9))
                                    .with_line_break_mode(LineBreaking::WordWrap)
                            )
                            .with_spacer(20.)
                            .with_child(
                                Flex::row()
                                    .must_fill_main_axis(true)
                                    .with_flex_spacer(1.)
                                    .with_child(Buttons::btn_ocr_form_close(false))
                                    .with_flex_spacer(1.)
                            )
                            .padding(20.)
                    )
                }
                _ => {
                    unreachable!()
                }
            }
        },
    )
}

enum OcrFormFields {
    NumLines,
    PageNum,
}

fn ocr_form(
    id: usize,
    data: &ApplicationState,
    field: OcrFormFields,
) -> impl Widget<ApplicationState> {
    macro_rules! mapping_lens {
        ($field: tt) => {
            lens!(ApplicationState, bookcase)
                .then(lens!(BookCase, library))
                .index(
                    data.bookcase
                        .library
                        .iter()
                        .position(|b| *b.path == data.book_to_align.get_path())
                        .unwrap(),
                )
                .then(lens!(BookInfo, ocr))
                .then(lens!(OcrData, mappings))
                .index(id)
                .then(lens!(Mapping, $field))
        };
    }

    match field {
        OcrFormFields::NumLines => Container::new(
            TextBox::new()
                .with_formatter(CustomFormatter::new())
                .update_data_while_editing(true)
                .lens(mapping_lens!(page_lines)),
        ),
        OcrFormFields::PageNum => Container::new(
            TextBox::new()
                .with_formatter(CustomFormatter::new())
                .update_data_while_editing(true)
                .lens(mapping_lens!(page)),
        ),
    }

}
