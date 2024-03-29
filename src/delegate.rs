use std::env::current_dir;
use std::fs;
use crate::app::{
    InputMode, FINISH_BOOK_LOAD, FINISH_IMAGE_LOAD, FINISH_LEPTO_LOAD, FINISH_SLOW_FUNCTION,
};
use crate::book::page_element::{ContentType, ImageState};
use crate::book::Book;
use crate::bookcase::BookInfo;
use crate::ocr::OcrData;
use crate::utilities::th_lepto_load;
use crate::ApplicationState;
use druid::commands::{OPEN_PANEL_CANCELLED, SAVE_PANEL_CANCELLED};
use druid::im::Vector;
use druid::{commands, AppDelegate, Command, DelegateCtx, Env, Handled, Target};
use std::path::PathBuf;

extern crate num_cpus;

pub(crate) struct Delegate {}

impl AppDelegate<ApplicationState> for Delegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut ApplicationState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::SAVE_FILE_AS) {
            let target_path = file_info.path.clone().to_str().unwrap().to_string();

            if let Err(e) = data.book_to_view.save(data.modified.clone(), target_path.clone()){
                data.error_message =
                    Option::Some("Impossible to save epub: ".to_string() + &e.to_string());

                let mut dir = current_dir().unwrap().to_str().unwrap().to_string();
                dir.push_str("/tmp/");
                let path_dir = PathBuf::from(&dir);
                if path_dir.is_dir() {
                    fs::remove_dir_all(&dir).unwrap();
                    println!("removed dir");
                }

                data.is_loading = false;
                return Handled::Yes;
            };
            data.modified.clear();

            let mut current = data.get_current_book_info().clone();
            //Il currentpath diventa quello del nuovo libro
            match data
                .bookcase
                .library
                .iter_mut()
                .find(|el| el.path == target_path.clone())
            {
                Some(b_info) => {
                    b_info.start_chapter = current.start_chapter;
                    b_info.start_element_number = current.start_element_number;
                    b_info.ocr = OcrData::new();
                    b_info.mapped_pages = Vector::new();
                    b_info.name = PathBuf::from(target_path.clone())
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    data.bookcase.update_meta();
                }
                None => {
                    current.path = target_path.clone();
                    current.name = PathBuf::from(target_path.clone())
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    current.ocr = OcrData::new();
                    current.mapped_pages = Vector::new();

                    data.bookcase.library.push_back(current);
                    data.bookcase.update_meta();
                }
            }
            data.set_book_to_read(Book::empty_book());
            data.edit = false;
            data.is_loading = false;
            return Handled::Yes;
        }

        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            match data.i_mode {
                InputMode::OcrJump | InputMode::OcrSyn0 | InputMode::OcrSyn1 => {

                    /* Qui stiamo prendendo un immagine per usare l'OCR */
                    th_lepto_load(
                        ctx.get_external_handle(),
                        file_info.path.clone(),
                        &data.get_current_book_info().language,
                    );
                }
                InputMode::EbookAdd => {
                    if data
                        .bookcase
                        .library
                        .iter()
                        .find(|el| el.path == file_info.path.clone().to_str().unwrap().to_string())
                        .is_some()
                    {
                        data.error_message = Some("Book already in library".to_string());
                    } else {
                        match BookInfo::new(file_info.path.clone().to_str().unwrap().to_string()) {
                            Ok(b) => {
                                data.bookcase.library.push_back(b);
                                data.bookcase.update_meta();
                            }
                            Err(_) => {
                                data.error_message =
                                    Option::Some("Impossible to open selected Epub".to_string());
                            }
                        }
                        data.i_mode = InputMode::None;
                    }
                    data.is_loading = false;
                }
                _ => (),
            }
            return Handled::Yes;
        }

        if let Some(res) = cmd.get(FINISH_SLOW_FUNCTION) {
            // If the command we received is `FINISH_SLOW_FUNCTION` handle the payload.
            if let Some((ch, off, str)) = res {
                match data.i_mode {
                    InputMode::OcrJump => {
                        data.book_to_view.get_mut_nav().set_ch(*ch);
                        data.update_view(ctx.get_external_handle());

                        data.book_to_view
                            .get_mut_nav()
                            .set_element_number(data.view.ocr_offset_to_element(*off));

                        if data.get_current_book_info().ocr.is_aligned() {
                            let _ = data
                                .get_mut_current_book_info()
                                .unwrap()
                                .ocr
                                .ocr_log(str.clone());
                        }
                    }
                    InputMode::OcrSyn0 => {
                        match data
                            .get_mut_current_book_info()
                            .unwrap()
                            .ocr
                            .ocr_log_first(str.clone(), *ch)
                        {
                            Ok(_) => data.view.ocr_form_stage = 3,
                            Err(_) => {
                                data.error_message = Some(
                                    "Image not recognized, please try with another image."
                                        .to_string(),
                                )
                            }
                        };
                    }
                    InputMode::OcrSyn1 => {
                        match data
                            .get_mut_current_book_info()
                            .unwrap()
                            .ocr
                            .ocr_log_other(str.clone())
                        {
                            Ok(_) => data.view.ocr_form_stage = 5,
                            Err(_) => {
                                data.error_message = Some(
                                    "Image not recognized, please try with another image."
                                        .to_string(),
                                )
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                data.error_message = Some(
                    "No matches were found, please try again with a better quality image."
                        .to_string(),
                );
                data.book_to_view = Book::empty_book();
            }
            data.i_mode = InputMode::None;
            data.is_loading = false;
            return Handled::Yes;
        }

        if let Some(str) = cmd.get(FINISH_LEPTO_LOAD) {
            match str {
                Some(str) => match data.i_mode {
                    InputMode::OcrJump => data.ocr_jump(ctx.get_external_handle(), str.to_string()),
                    InputMode::OcrSyn0 => data.ocr_jump(ctx.get_external_handle(), str.to_string()),
                    InputMode::OcrSyn1 => data.ocr_jump(ctx.get_external_handle(), str.to_string()),
                    _ => {}
                },
                None => {
                    data.error_message = Some("Couldn't load image".to_string());
                    data.book_to_view = Book::empty_book();
                }
            }
            return Handled::Yes;
        }

        if let Some(book) = cmd.get(FINISH_BOOK_LOAD) {
            match book {
                Some(book) => {
                    data.set_book_to_read(book.clone());
                    data.update_view(ctx.get_external_handle());
                }
                None => {
                    data.error_message = Some("Couldn't load book".to_string());
                    data.book_to_view = Book::empty_book();
                }
            }
            data.is_loading = false;
            return Handled::Yes;
        }

        if let Some((img, path)) = cmd.get(FINISH_IMAGE_LOAD) {
            if let Some(element) =
                data.view
                    .current_view
                    .iter_mut()
                    .find(|el| match el.content.clone() {
                        ContentType::Image(ImageState::Waiting(str)) => str == *path,
                        _ => false,
                    })
            {
                element.content = ContentType::Image(ImageState::Present(img.clone()));
                if !data.book_to_view.is_empty() {
                    data.book_to_view
                        .imgs
                        .entry(path.clone())
                        .or_insert(img.clone());
                }
                // data.update_view(ctx.get_external_handle());
            }
            return Handled::Yes;
        }

        if let Some(..) = cmd.get(SAVE_PANEL_CANCELLED) {
            data.is_loading = false;
            return Handled::Yes;
        }

        if let Some(..) = cmd.get(OPEN_PANEL_CANCELLED) {
            data.book_to_view = Book::empty_book();
            match data.i_mode {
                InputMode::EbookAdd => {
                    data.book_to_view = Book::empty_book();
                    data.i_mode = InputMode::None;
                }
                InputMode::OcrJump => data.i_mode = InputMode::None,
                InputMode::OcrSyn1 | InputMode::OcrSyn0 => data.i_mode = InputMode::None,
                _ => {}
            }
            data.is_loading = false;
            return Handled::Yes;
        }

        Handled::No
    }
}
