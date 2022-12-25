use std::fmt::Alignment;
use crate::book::{chapter::Chapter, Book};
use crate::controllers::{Update, BetterScroll, SyncScroll};
use crate::view::buttons::Buttons;
use crate::view::view::View;
use crate::{ApplicationState, PageElement};
use druid::widget::{Axis, Painter, SvgData, Svg, Scroll, ControllerHost, Click, CrossAxisAlignment, FillStrat, Flex, FlexParams, Image, LineBreaking, List, RawLabel, Spinner, TextBox, ViewSwitcher, ClipBox, Button, Align, LabelText, Label, Container, Padding, SizedBox};
use druid::{lens, ImageBuf, LensExt, Widget, WidgetExt, Vec2, LifeCycle, Selector, FileDialogOptions, FileSpec, Color, KeyOrValue, RenderContext};
use druid::Cursor::Custom;
use druid::keyboard_types::Key::Control;
use crate::utilities::save_file;

//SWITCH TRA VISUALIZZATORE ELENCO EBOOK E VISUALIZZATORE EBOOK
pub fn build_main_view() -> impl Widget<ApplicationState> {
    let main_nav: ViewSwitcher<ApplicationState, bool> =
        ViewSwitcher::new(
            |data: &ApplicationState, _| data.is_loading, /* Ad ora non funziona... lo fixo */
            |_load, data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>> {
                if !data.is_loading {
                    Box::new(
                        ViewSwitcher::new(
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
                        data.xml_backup == data.current_book.chapters[data.current_book.get_nav().get_ch()].xml
                    },
                    |cond, _data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
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
                let mut buttons: Flex<ApplicationState> = Flex::row();
                buttons.add_child(Buttons::btn_prev());
                buttons.add_child(Buttons::btn_edit());
                buttons.add_child(Buttons::btn_save());
                buttons.add_child(Buttons::btn_close_book());
                buttons.add_child(Buttons::btn_next());
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
                Update::new(|_, data: &mut ApplicationState, _| {
                    data.update_view()
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
                }).lens(lens);
            viewport.add_child(chapter);
            Box::new(viewport.padding(30.0))
        })
}

fn render_library() -> impl Widget<ApplicationState> {
    ViewSwitcher::new(
        |data: &ApplicationState, _| data.bookcase.library.clone(),
        |_app, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>>
            { // TODO:Load IMAGES IN THREAD
                let mut col = Flex::column();
                col.add_spacer(12.0);
                for (i, book_info) in data.get_library().clone().into_iter().enumerate() {
                    let mut pill = Flex::row().cross_axis_alignment(CrossAxisAlignment::Start);
                    let mut uno = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start)
                        .with_child(
                            Image::new(ImageBuf::from_file(book_info.cover_path.clone())
                                .unwrap()) //TODO: unwrap_or(default image)
                                .fix_width(300.0)
                                .fix_height(200.0)
                        );
                    let mut due = Flex::column()
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .with_spacer(15.0)
                        .with_child(Label::new(String::from("Title: ") + &*book_info.name.clone()).with_text_size(20.0))
                        .with_spacer(4.0)
                        .with_child(Label::new(String::from("Chapter: ") + &*book_info.start_chapter.clone().to_string()))
                        .with_spacer(1.0)
                        .with_child(Label::new(String::from("Offset: ") + &*book_info.start_line.clone().to_string()))
                        .with_spacer(30.0)
                        .with_child(Flex::row()
                            .cross_axis_alignment(CrossAxisAlignment::Start)
                            .with_child(
                                Buttons::btn_ocr(book_info.clone())
                            )
                            .with_spacer(10.0)
                            .with_child(Buttons::btn_remove_book(book_info.path.clone()))
                            .with_spacer(10.0)
                            .with_child(Buttons::btn_read_book(book_info.clone()))
                        );

                    pill.add_flex_child(Padding::new((0.0,2.0,10.0,2.0),uno), 0.3);
                    pill.add_flex_child(due, 0.7);


                    /*let wrap = Container::new(pill)
                        .border(Color::WHITE, 1.0)
                        .rounded(8.0);*/

                    col.add_child(Padding::new((12.0, 0.0, 12.0, 8.0),pill));
                    if i != (data.get_library().len() - 1) {
                        col.add_child(
                            Painter::new(|ctx, _: &_, _: &_| {
                                let size = ctx.size().to_rect();
                                ctx.fill(size, &Color::WHITE)
                            })
                                .fix_height(1.0).padding(20.0),
                        );
                    }

                }
                Box::new(col.scroll().vertical())
            },
    )
}


/*
fn render_prova() -> impl Widget<ApplicationState> {
    let rs = FileSpec::new("Rust source", &["rs"]);
    let txt = FileSpec::new("Text file", &["txt"]);
    let other = FileSpec::new("Bogus file", &["foo", "bar", "baz"]);
    // The options can also be generated at runtime,
    // so to show that off we create a String for the default save name.
    let default_save_name = String::from("MyFile.txt");
    let save_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![rs, txt, other])
        .default_type(txt)
        .default_name(default_save_name)
        .name_label("Target")
        .title("Choose a target for this lovely file")
        .button_text("Export");
    let open_dialog_options = save_dialog_options
        .clone()
        .default_name("MySavedFile.txt")
        .name_label("Source")
        .title("Where did you put that file?")
        .button_text("Import");

    let input = TextBox::new().lens(ApplicationState::buffer);
    let save = Button::new("Save").on_click(move |ctx, data:&mut ApplicationState, _| {
        ctx.submit_command(druid::commands::SHOW_SAVE_PANEL.with(save_file(data.get_current().name.clone())));
    });
    let open = Button::new("Open").on_click(move |ctx, _, _| {
        ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options.clone()))
    });

    let mut col = Flex::column();
    col.add_child(input);
    col.add_spacer(8.0);
    col.add_child(save);
    col.add_child(open);
    Align::centered(col)
}
*/