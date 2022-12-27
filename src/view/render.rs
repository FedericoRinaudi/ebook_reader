use crate::book::{chapter::Chapter, Book};
use crate::controllers::Update;
use crate::view::buttons::Buttons;
use crate::view::view::View;
use crate::widgets::custom_scrolls::{BetterScroll, SyncScroll};
use crate::{ApplicationState, ContentType};
use druid::widget::{
    ControllerHost, CrossAxisAlignment, FillStrat, Flex, FlexParams, Image, Label, LineBreaking,
    List, Padding, Painter, RawLabel, Scroll, Spinner, TextBox, ViewSwitcher,
};
use druid::{lens, Color, ImageBuf, LensExt, RenderContext, Widget, WidgetExt};
use crate::book::page_element::PageElement;
use crate::widgets::custom_label::BetterLabel;

//SWITCH TRA VISUALIZZATORE ELENCO EBOOK E VISUALIZZATORE EBOOK
pub fn build_main_view() -> impl Widget<ApplicationState> {
    let main_nav: ViewSwitcher<ApplicationState, bool> = ViewSwitcher::new(
        |data: &ApplicationState, _| data.is_loading, /* Ad ora non funziona... lo fixo */
        |_load, data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>> {
            if !data.is_loading {
                Box::new(ViewSwitcher::new(
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
                ))
            } else {
                Box::new(Spinner::new().fix_height(40.0).center())
            }
        },
    );

    main_nav
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
                            == data.current_book.chapters[data.current_book.get_nav().get_ch()].xml
                    },
                    |cond, _data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
                        Box::new( if *cond {
                            Flex::row()
                                .with_child(Buttons::bnt_view())
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
                let mut buttons: Flex<ApplicationState> = Flex::row()
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
                window.add_flex_child(screen, 0.9);
                window.add_child(Flex::row().fix_height(1.0));
                Box::new(window)
            }
        },
    )
}

fn render_edit_mode() -> impl Widget<ApplicationState> {
    let mut viewport = Flex::row();
    let view = ViewSwitcher::new(
        |data: &ApplicationState, _env| {
            data.current_book.chapters[data.current_book.get_nav().get_ch()]
                .xml
                .clone()
        },
        |_vec, _data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            Box::new(BetterScroll::new(render_view_mode()))
        },
    );
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
                                Box::new(BetterLabel::new())
                            },
                            ContentType::Image(img_buf) => Box::new(Flex::row().with_child(
                                Image::new(img_buf.clone()).fill_mode(FillStrat::ScaleDown),
                            )),
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
            Box::new(Padding::new((30.0,0.0,30.0,0.0),viewport))
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
                        .with_child(Buttons::btn_add_book().padding(20.0)),
                );
            //TODO: provo con molti libri e valuto le tempistiche, valuto multithread
            for (i, book_info) in data.get_library().clone().into_iter().enumerate() {
                let mut pill = Flex::row().cross_axis_alignment(CrossAxisAlignment::Start);
                let uno = Flex::column()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_child(
                        Image::new(ImageBuf::from_file(book_info.cover_path.clone()).unwrap()) //TODO: unwrap_or(default image)
                            .fix_width(300.0)
                            .fix_height(200.0),
                    );
                let due = Flex::column()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_spacer(15.0)
                    .with_child(Label::new(&*book_info.name.clone()).with_text_size(20.0).with_line_break_mode(LineBreaking::WordWrap))
                    .with_spacer(4.0)
                    //TODO: Salvo su file e aggiungo le informazioni corrette
                    .with_child(Label::new(
                        String::from("Chapter: ") + &*book_info.start_chapter.clone().to_string(),
                    ))
                    .with_spacer(1.0)
                    .with_child(Label::new(
                        String::from("Offset: ") + &*book_info.start_element_number.clone().to_string(),
                    ))
                    .with_spacer(90.0)
                    .with_child(
                        Flex::row()
                            .cross_axis_alignment(CrossAxisAlignment::Start)
                            .with_child(Buttons::btn_ocr(book_info.clone()))
                            .with_spacer(10.0)
                            .with_child(Buttons::btn_remove_book(i))
                            .with_spacer(10.0)
                            .with_child(Buttons::btn_read_book(book_info.clone())),
                    );

                pill.add_flex_child(Padding::new((0.0, 2.0, 10.0, 2.0), uno), 0.3);
                pill.add_flex_child(Padding::new((0.0, 0.0, 0.0, 10.0), due), 0.7);

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
