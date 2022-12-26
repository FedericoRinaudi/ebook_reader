use crate::bookcase::BookInfo;
use crate::utilities::{open_epub, open_image, save_file};
use crate::{ApplicationState, Book};
use druid::widget::{
    Align, Button, Click, ControllerHost, DisabledIf, Flex, Svg, SvgData, ViewSwitcher,
};
use druid::{Widget, WidgetExt};
use std::fs;
//use crate::controllers::ClickableOpacity;
const LIBRARY_SVG_DIM: f64 = 30.;
const LIBRARY_SVG_BIG: f64 = 35.;

pub struct Buttons {}

impl Buttons {
    pub fn btn_next() -> ControllerHost<Align<ApplicationState>, Click<ApplicationState>> {
        let right_svg = match include_str!("../../icons/right.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(right_svg.clone())
            .fix_width(LIBRARY_SVG_BIG)
            .center()
            .on_click(|_ctx, data: &mut ApplicationState, _env| {
                data.current_book.go_on(1);
                data.update_view()
            })
    }

    pub fn btn_prev() -> ControllerHost<Align<ApplicationState>, Click<ApplicationState>> {
        let left_svg = match include_str!("../../icons/left.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(left_svg.clone())
            .fix_width(LIBRARY_SVG_BIG)
            .center()
            .on_click(|_ctx, data: &mut ApplicationState, _env| {
                data.current_book.go_back(1);
                data.update_view()
            })
    }

    pub fn btn_confirm() -> DisabledIf<
        ApplicationState,
        ControllerHost<Button<ApplicationState>, Click<ApplicationState>>,
    > {
        Button::new("Confirm")
            .on_click(|_ctx, data: &mut ApplicationState, _env| {
                /* EDIT MODE -> EDIT MODE, CONFIRM CHANGES */
                // data.current_book.save();
                data.xml_backup = data.current_book.chapters[data.current_book.get_nav().get_ch()].xml.clone();
                data.modified.insert(data.current_book.get_nav().get_ch()); /* Inserisco se non è già presente il capitolo corrente in quelli modificati */
            }).disabled_if(|data: &ApplicationState, _| data.view.current_view.len()==0 || data.view.current_view[0].is_err())
    }

    pub fn btn_edit() -> ControllerHost<Align<ApplicationState>, Click<ApplicationState>> {
        let edit_svg = match include_str!("../../icons/edit.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(edit_svg.clone())
            .fix_width(LIBRARY_SVG_DIM)
            .center()
            .on_click(|ctx, data: &mut ApplicationState, _env| {
                /*  VIEW MODE -> EDIT MODE */
                data.xml_backup = data.current_book.chapters[data.current_book.get_nav().get_ch()]
                    .xml
                    .clone();
                data.view
                    .set_window_size_view(<(f64, f64)>::from(ctx.window().get_size()));
                ctx.window().set_size(data.view.get_window_size_edit());
                ctx.window().set_title("EDIT MODE");

                data.edit = !data.edit
            })
    }

    pub fn bnt_view() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {
        Button::new("View").on_click(|ctx, data: &mut ApplicationState, _env| {
            /*  EDIT MODE -> VIEW MODE */
            data.view
                .set_window_size_edit(<(f64, f64)>::from(ctx.window().get_size()));
            ctx.window().set_size(data.view.get_window_size_view());
            ctx.window().set_title("VIEW MODE");
            data.edit = !data.edit;
        })
    }

    pub fn bnt_discard() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {
        Button::new("Discard").on_click(|_ctx, data: &mut ApplicationState, _env| {
            /* EDIT MODE -> EDIT MODE, Discard Changes */
            data.current_book.update_xml(data.xml_backup.clone());
            data.update_view();
        })
    }

    //TODO: Button to save on file
    pub fn btn_save() -> ViewSwitcher<ApplicationState, bool> {
        ViewSwitcher::new(
            |data: &ApplicationState, _| data.modified.is_empty(),
            |cond, _data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
                match !cond {
                    true => {
                        let save_svg = match include_str!("../../icons/save.svg").parse::<SvgData>()
                        {
                            Ok(svg) => svg,
                            Err(_) => SvgData::default(),
                        };
                        Box::new(Svg::new(save_svg)
                            .fix_width(LIBRARY_SVG_DIM)
                            .center()
                            .on_click(|ctx, data: &mut ApplicationState, _env| {
                                /* SAVE CHANGES ON NEW FILE */
                                ctx.submit_command(druid::commands::SHOW_SAVE_PANEL.with(
                                    save_file(
                                        data.get_current().name.clone() + &*String::from(".epub"),
                                    ),
                                ));
                            })
                        )
                    }
                    false => {
                        let save_disabled_svg = match include_str!("../../icons/save_disabled.svg")
                            .parse::<SvgData>()
                        {
                            Ok(svg) => svg,
                            Err(_) => SvgData::default(),
                        };
                        Box::new(Svg::new(save_disabled_svg)
                            .fix_width(LIBRARY_SVG_DIM)
                            .center()
                        )
                    }
                }
            },
        )
    }

    pub fn btn_ocr(book_info: BookInfo) -> impl Widget<ApplicationState> {
        let ocr_svg = match include_str!("../../icons/ocr.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(ocr_svg)
            .fix_width(LIBRARY_SVG_DIM)
            .center()
            .on_click(move |ctx, data: &mut ApplicationState, _env| {
                /* Tries to load image and find matching line in chapter */
                data.i_mode = true;
                data.is_loading = true;
                data.current_book = Book::new(
                    book_info.get_path(),
                    book_info.start_chapter,
                    book_info.start_line,
                )
                .unwrap();
                ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_image()));
            })
    }

    pub fn btn_close_book() -> ControllerHost<Align<ApplicationState>, Click<ApplicationState>> {
        let library_svg = match include_str!("../../icons/library.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(library_svg.clone())
            .fix_width(LIBRARY_SVG_BIG)
            .center()
            .on_click(|_ctx, data: &mut ApplicationState, _env| data.close_current_book())
    }

    pub fn btn_remove_book(index: usize) -> impl Widget<ApplicationState> {
        let trash_bin_svg = match include_str!("../../icons/trash_bin.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(trash_bin_svg.clone())
            .fix_width(LIBRARY_SVG_DIM)
            .center()
            .on_click(move |_, data: &mut ApplicationState, _| {
                let removed_book_info = data.bookcase.library.remove(index);
                data.bookcase.update();
                //TODO: anzi che le print metto poi un pop-up o comunque do un feedback all'utente
                match fs::remove_file(removed_book_info.path.clone()) {
                    Ok(()) => println!("Successfully removed file"),
                    Err(e) => println!("Error deleting file: {}", e),
                }
            })
    }

    pub fn btn_add_book() -> impl Widget<ApplicationState> {
        let add_svg = match include_str!("../../icons/add.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(add_svg.clone())
            .fix_width(LIBRARY_SVG_BIG)
            .center()
            .on_click(|ctx, _, _| {
                ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_epub()));
            })
    }

    pub fn btn_read_book(book_info: BookInfo) -> impl Widget<ApplicationState> {
        let book_svg = match include_str!("../../icons/read.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(book_svg.clone())
            .fix_width(LIBRARY_SVG_DIM)
            .center()
            .on_click(move |_ctx, data: &mut ApplicationState, _env| {
                println!("{}", book_info.path.clone());
                data.current_book = Book::new(
                    book_info.get_path(),
                    book_info.start_chapter,
                    book_info.start_line,
                )
                .unwrap();
                data.update_view();
            })
    }
}
