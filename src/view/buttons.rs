use crate::app::InputMode;
use crate::bookcase::BookInfo;
use crate::ocr::OcrData;
use crate::utilities::{open_epub, open_image, save_file, th_load_book};
use crate::widgets::custom_tooltip::TipExt;
use crate::{ApplicationState, Book};
use druid::im::Vector;
use druid::widget::{Align, Button, Click, ControllerHost, Svg, SvgData, ViewSwitcher};
use druid::{Env, Widget, WidgetExt};

//use crate::controllers::ClickableOpacity;
const LIBRARY_SVG_DIM: f64 = 30.;
const LIBRARY_SVG_BIG: f64 = 35.;

pub struct Buttons {}

impl Buttons {
    pub fn btn_next() -> impl Widget<ApplicationState> {
        let right_svg = match include_str!("../../icons/right.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(right_svg.clone())
            .fix_width(LIBRARY_SVG_BIG)
            .center()
            .on_click(|_ctx, data: &mut ApplicationState, _env| {
                data.book_to_view.go_on(1);
                data.update_view()
            })
            .tooltip(
                |_data: &ApplicationState, _env: &Env| "Next Page".to_string(),
                false,
            )
    }

    pub fn btn_prev() -> impl Widget<ApplicationState> {
        let left_svg = match include_str!("../../icons/left.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(left_svg.clone())
            .fix_width(LIBRARY_SVG_BIG)
            .center()
            .on_click(|_ctx, data: &mut ApplicationState, _env| {
                data.book_to_view.go_back(1);
                data.update_view()
            })
            .tooltip(
                |_data: &ApplicationState, _env: &Env| "Prev Page".to_string(),
                false,
            )
    }

    pub fn btn_confirm() -> impl Widget<ApplicationState> {
        let confirm_svg = match include_str!("../../icons/confirm.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        let confirm_disabled_svg =
            match include_str!("../../icons/confirm_disabled.svg").parse::<SvgData>() {
                Ok(svg) => svg,
                Err(_) => SvgData::default(),
            };

        ViewSwitcher::new(
            move |data: &ApplicationState, _| {
                !(data.view.current_view.len() == 0 || data.view.current_view[0].content.is_err())
            },
            move |cond, _data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
                match cond {
                    true => {
                        Box::new(
                            Svg::new(confirm_svg.clone())
                                .fix_width(LIBRARY_SVG_DIM)
                                .center()
                                .on_click(|_ctx, data: &mut ApplicationState, _env| {
                                    /* EDIT MODE -> EDIT MODE, CONFIRM CHANGES */
                                    // data.current_book.save();
                                    data.xml_backup = data.book_to_view.chapters[data.book_to_view.get_nav().get_ch()].xml.clone();
                                    data.modified.insert(data.book_to_view.get_nav().get_ch()); /* Inserisco se non è già presente il capitolo corrente in quelli modificati */
                                })
                                .tooltip(|_data:&ApplicationState, _env: &Env| "Confirm Changes locally".to_string(), false)
                        )
                    }
                    false => Box::new(
                        Svg::new(confirm_disabled_svg.clone())
                            .fix_width(LIBRARY_SVG_DIM)
                            .center(),
                    ),
                }
            },
        )
    }

    pub fn btn_edit() -> impl Widget<ApplicationState> {
        let edit_svg = match include_str!("../../icons/edit.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(edit_svg.clone())
            .fix_width(LIBRARY_SVG_DIM)
            .center()
            .on_click(|ctx, data: &mut ApplicationState, _env| {
                /*  VIEW MODE -> EDIT MODE */
                data.xml_backup = data.book_to_view.chapters[data.book_to_view.get_nav().get_ch()]
                    .xml
                    .clone();
                data.view
                    .set_window_size_view(<(f64, f64)>::from(ctx.window().get_size()));
                ctx.window().set_size(data.view.get_window_size_edit());
                ctx.window().set_title("EDIT MODE");

                data.edit = !data.edit
            })
            .tooltip(
                |_data: &ApplicationState, _env: &Env| "Edit XML".to_string(),
                false,
            )
    }

    pub fn btn_view() -> impl Widget<ApplicationState> {
        let read_svg = match include_str!("../../icons/read.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(read_svg.clone())
            .fix_width(LIBRARY_SVG_BIG)
            .center()
            .on_click(|ctx, data: &mut ApplicationState, _env| {
                /*  EDIT MODE -> VIEW MODE */
                data.view
                    .set_window_size_edit(<(f64, f64)>::from(ctx.window().get_size()));
                ctx.window().set_size(data.view.get_window_size_view());
                ctx.window().set_title("VIEW MODE");
                data.edit = !data.edit;
            })
            .tooltip(
                |_data: &ApplicationState, _env: &Env| "Back to View Mode".to_string(),
                false,
            )
    }

    pub fn btn_ocr_syn(book_info_id: usize) -> ViewSwitcher<ApplicationState, bool> {
        let align_svg = match include_str!("../../icons/align.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        let misalign_svg = match include_str!("../../icons/misalign.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        ViewSwitcher::new(
            move |data: &ApplicationState, _| {
                data.bookcase.library[book_info_id].mapped_pages.is_empty()
            },
            move |cond, _data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
                match cond {
                    true => Box::new(
                        Svg::new(align_svg.clone())
                            .fix_width(LIBRARY_SVG_DIM)
                            .center()
                            .on_click(move |_ctx, data: &mut ApplicationState, _env| {
                                let book_info = data.bookcase.library[book_info_id].clone();
                                data.set_book_to_align(
                                    Book::new(
                                        book_info.get_path(),
                                        book_info.start_chapter,
                                        book_info.start_element_number,
                                        &book_info.mapped_pages,
                                    )
                                    .unwrap(),
                                );
                            })
                            .tooltip(
                                |_data: &ApplicationState, _env: &Env| {
                                    "Phisical book alignment".to_string()
                                },
                                false,
                            ),
                    ),
                    false => Box::new(
                        Svg::new(misalign_svg.clone())
                            .fix_width(LIBRARY_SVG_DIM)
                            .center()
                            .on_click(move |_ctx, data: &mut ApplicationState, _env| {
                                let mut book_info = &mut data.bookcase.library[book_info_id];
                                book_info.ocr = OcrData::new();
                                book_info.mapped_pages = Vector::new();
                                data.bookcase.update_meta();
                            })
                            .tooltip(
                                |_data: &ApplicationState, _env: &Env| {
                                    "Remove alignment".to_string()
                                },
                                false,
                            ),
                    ),
                }
            },
        )
    }

    pub fn btn_discard() -> impl Widget<ApplicationState> {
        let discard_svg = match include_str!("../../icons/discard.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(discard_svg.clone())
            .fix_width(LIBRARY_SVG_BIG)
            .center()
            .on_click(|_ctx, data: &mut ApplicationState, _env| {
                /* EDIT MODE -> EDIT MODE, Discard Changes */
                data.book_to_view.update_xml(data.xml_backup.clone());
                data.update_view();
            })
            .tooltip(
                |_data: &ApplicationState, _env: &Env| "Discard changes".to_string(),
                false,
            )
    }

    pub fn btn_close_error() -> ControllerHost<Align<ApplicationState>, Click<ApplicationState>> {
        let close_svg = match include_str!("../../icons/close_error.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(close_svg.clone())
            .fix_width(20.)
            .center()
            .on_click(|_ctx, data: &mut ApplicationState, _env| {
                /* EDIT MODE -> EDIT MODE, Discard Changes */
                data.error_message = None;
            })
    }

    //TODO: Button to save on file
    pub fn btn_save() -> ViewSwitcher<ApplicationState, bool> {
        //VIWE SWITCHER PER DISABILITARE O MENO L'ICONA
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
                        Box::new(
                            Svg::new(save_svg)
                                .fix_width(LIBRARY_SVG_DIM)
                                .center()
                                .on_click(|ctx, data: &mut ApplicationState, _env| {
                                    /* SAVE CHANGES ON NEW FILE */
                                    ctx.submit_command(druid::commands::SHOW_SAVE_PANEL.with(
                                        save_file(
                                            data.get_current_book_info().name.clone()
                                                + &*String::from(".epub"),
                                        ),
                                    ));
                                })
                                .tooltip(
                                    |_data: &ApplicationState, _env: &Env| {
                                        "Save on file".to_string()
                                    },
                                    false,
                                ),
                        )
                    }
                    false => {
                        let save_disabled_svg = match include_str!("../../icons/save_disabled.svg")
                            .parse::<SvgData>()
                        {
                            Ok(svg) => svg,
                            Err(_) => SvgData::default(),
                        };
                        Box::new(
                            Svg::new(save_disabled_svg)
                                .fix_width(LIBRARY_SVG_DIM)
                                .center(),
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
                data.i_mode = InputMode::OcrJump;
                data.is_loading = true;
                data.set_book_to_read(
                    Book::new(
                        book_info.get_path(),
                        book_info.start_chapter,
                        book_info.start_element_number,
                        &book_info.mapped_pages,
                    )
                    .unwrap(),
                );
                ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_image()));
            })
            .tooltip(
                |_data: &ApplicationState, _env: &Env| "Jump to photo".to_string(),
                false,
            )
    }

    pub fn btn_close_book() -> impl Widget<ApplicationState> {
        let library_svg = match include_str!("../../icons/library.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(library_svg.clone())
            .fix_width(LIBRARY_SVG_BIG)
            .center()
            .on_click(|_ctx, data: &mut ApplicationState, _env| data.close_current_book())
            .tooltip(
                |_data: &ApplicationState, _env: &Env| "Close Book".to_string(),
                false,
            )
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
                let _ = data.bookcase.library.remove(index);
                data.bookcase.update_meta();
            })
            .tooltip(
                |_data: &ApplicationState, _env: &Env| "Remove book from library".to_string(),
                false,
            )
    }

    pub fn btn_add_book() -> impl Widget<ApplicationState> {
        let add_svg = match include_str!("../../icons/add.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(add_svg.clone())
            .fix_width(LIBRARY_SVG_BIG)
            .center()
            .on_click(|ctx, data: &mut ApplicationState, _| {
                data.i_mode = InputMode::EbookAdd;
                ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_epub()));
            })
            .tooltip(
                |_data: &ApplicationState, _env: &Env| "Add book to library".to_string(),
                false,
            )
    }

    pub fn btn_read_book(book_info: BookInfo) -> impl Widget<ApplicationState> {
        let book_svg = match include_str!("../../icons/read.svg").parse::<SvgData>() {
            Ok(svg) => svg,
            Err(_) => SvgData::default(),
        };
        Svg::new(book_svg.clone())
            .fix_width(LIBRARY_SVG_DIM)
            .center()
            .on_click(move |ctx, data: &mut ApplicationState, _env| {
                data.is_loading = true;
                th_load_book(
                    ctx.get_external_handle(),
                    book_info.get_path(),
                    book_info.start_chapter,
                    book_info.start_element_number,
                    book_info.mapped_pages.clone(),
                );
            })
            .tooltip(
                |_data: &ApplicationState, _env: &Env| "Read the book".to_string(),
                false,
            )
    }

    pub fn btn_submit_ocr_form0(
    ) -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {
        Button::new("CONFIRM").on_click(|_ctx, data: &mut ApplicationState, _env| {
            let book_info = data.get_mut_current_book_info().unwrap();
            if (*book_info).ocr.first.is_none() {
                data.error_message = Option::Some("You have to fill all felds.".to_string());
            } else {
                let first = &(*book_info).ocr.mappings[(*book_info).ocr.first.unwrap()];
                if (*first).page_lines != 0 && (*first).page != 0 {
                    data.view.ocr_form_stage += 1;
                } else {
                    data.error_message = Option::Some("You have to fill all felds.".to_string());
                }
            }
        })
    }

    //TODO
    pub fn btn_submit_ocr_form1(
    ) -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {

        Button::new("NEXT").on_click(|_ctx, data: &mut ApplicationState, _env| {
            let ocr = &data.get_current_book_info().ocr;
            if (*ocr).other.is_none() {
                data.error_message = Option::Some("You have to fill all felds.".to_string());
            } else {
                let other = &(*ocr).mappings[(*ocr).other.unwrap()];
                if  (*other).page_lines != 0
                {
                    let _ = data.map_pages(true);
                    data.bookcase.update_meta();
                    data.book_to_align = Book::empty_book();
                    data.view.ocr_form_stage = 6;
                } else {
                    data.error_message = Option::Some("You have to fill all felds.".to_string());
                }
            }
        })
    }

    pub fn btn_ocr_form_close() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>>
    {
        Button::new("LIBRARY").on_click(|_ctx, data: &mut ApplicationState, _env| {
            data.view.ocr_form_stage = 1;
            (*data.get_mut_current_book_info().unwrap()).ocr = OcrData::new();
            data.book_to_align = Book::empty_book();
        })
    }

    pub fn btn_ocr_form_next() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>>
    {
        Button::new("NEXT").on_click(|_ctx, data: &mut ApplicationState, _env| {
            data.view.ocr_form_stage += 1;
        })
    }

    pub fn btn_ocr_form_prev() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>>
    {
        Button::new("GO BACK").on_click(|_ctx, data: &mut ApplicationState, _env| {
            data.view.ocr_form_stage -= 1;
        })
    }

    pub fn btn_add_first_page() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>>
    {
        Button::new("LOAD PAGE").on_click(|ctx, data: &mut ApplicationState, _env| {
            data.i_mode = InputMode::OcrSyn0;
            ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_image()));
            data.is_loading = true;
        })
    }

    pub fn btn_add_other_page() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>>
    {
        Button::new("LOAD PAGE").on_click(|ctx, data: &mut ApplicationState, _env| {
            data.i_mode = InputMode::OcrSyn1;
            ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_image()));
        })
    }

    pub fn btn_remove_first_page(
    ) -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {
        Button::new("GO BACK").on_click(|_ctx, data: &mut ApplicationState, _env| {
            let ocr = &mut data.get_mut_current_book_info().unwrap().ocr;
            ocr.mappings.remove((*ocr).first.unwrap());
            ocr.first = None;
            data.view.ocr_form_stage = 2;
        })
    }

    pub fn btn_remove_other_page(
    ) -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {
        Button::new("REMOVE").on_click(|_ctx, data: &mut ApplicationState, _env| {
            let ocr = &mut data.get_mut_current_book_info().unwrap().ocr;
            ocr.mappings.remove((*ocr).other.unwrap());
            ocr.other = None;
            data.view.ocr_form_stage = 4;
        })
    }
}
