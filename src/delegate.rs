use crate::app::{InputMode, FINISH_LEPTO_LOAD, FINISH_SLOW_FUNCTION};
use crate::book::Book;
use crate::bookcase::{BookCase, BookInfo};
use crate::utilities::th_lepto_load;
use crate::ApplicationState;
use druid::commands::{OPEN_PANEL_CANCELLED, SAVE_PANEL_CANCELLED};
use druid::{commands, AppDelegate, Command, DelegateCtx, Env, Handled, Target};
use epub::doc::EpubDoc;
use std::path::PathBuf;
use std::{env, fs};

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
            let cwd = env::current_dir().unwrap();
            let absolute_path = file_info.path.clone();

            // "Normalizziamo" la path: Se fa riferimento a qualcosa nella nostra cartella la teniamo relativa
            let target_path = match absolute_path.clone().strip_prefix(cwd.clone()) {
                Ok(path) => "./".to_string() + path.to_str().unwrap(),
                Err(_e) => {
                    //eprintln!("Error stripping prefix from path {}", e);
                    absolute_path.clone().to_str().unwrap().to_string()
                }
            };

            data.book_to_view
                .save(data.modified.clone(), target_path.clone());
            data.modified.clear();

            //Il currentpath diventa quello del nuovo libro
            if data.book_to_view.get_path() != target_path {
                let mut copy_info: BookInfo = data.get_current_book_info();
                copy_info.path = target_path.clone();
                copy_info.name = PathBuf::from(target_path.clone())
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                data.book_to_view.path = target_path;
                data.bookcase.library.push_back(copy_info);
                data.bookcase.update();
            }
            return Handled::Yes;
        }

        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            match data.i_mode {
                InputMode::OcrJump | InputMode::OcrSyn0 | InputMode::OcrSyn1 => {
                    /* Qui stiamo prendendo un immagine per usare l'OCR */
                    th_lepto_load(ctx.get_external_handle(), file_info.path.clone());
                }
                InputMode::EbookAdd => {
                    if EpubDoc::new(file_info.path.clone()).is_ok() && file_info.path.is_file() {
                        data.is_loading = true;
                        fs::copy(
                            file_info.path.clone(),
                            "./libri/".to_owned()
                                + file_info.path.file_name().unwrap().to_str().unwrap(),
                        )
                        .expect("Failed to copy file");
                        data.bookcase = BookCase::new();
                        data.is_loading = false;
                    } else {
                        data.error_message =
                            Option::Some("Impossible to open selected Epub".to_string());
                    }
                    data.i_mode = InputMode::None;
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
                        data.update_view();

                        data.book_to_view
                            .get_mut_nav()
                            .set_element_number(data.view.ocr_offset_to_element(*off));

                        if data.get_current_book_info().ocr.is_aligned() {
                            let _ = data.get_mut_current_book_info()
                                .unwrap()
                                .ocr
                                .ocr_log(str.clone());
                        }
                    },
                    InputMode::OcrSyn0 => {
                        let _ = data.get_mut_current_book_info()
                            .unwrap()
                            .ocr
                            .ocr_log_first(str.clone());
                    },
                    InputMode::OcrSyn1 => {
                        let _ = data.get_mut_current_book_info()
                            .unwrap()
                            .ocr
                            .ocr_log_other(str.clone());
                    },
                    _=> {}
                }
                    }
             else {
                 println!("CIAOOOO {:?}", res);
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
                    InputMode::OcrJump => data.ocr_jump(
                        ctx.get_external_handle(),
                        str.to_string(),
                    ),
                    InputMode::OcrSyn0 => data.ocr_jump(
                        ctx.get_external_handle(),
                        str.to_string()),
                    InputMode::OcrSyn1 => data.ocr_jump(
                        ctx.get_external_handle(),
                        str.to_string()),
                    _ => {}
                },
                None => {
                    data.error_message = Some("Couldn't load image".to_string());
                    data.book_to_view = Book::empty_book();
                }
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
                _ => {},
            }
            data.is_loading = false;
            return Handled::Yes;
        }

        Handled::No
    }
}
