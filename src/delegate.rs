use crate::app::{FINISH_LEPTO_LOAD, FINISH_SLOW_FUNCTION};
use crate::book::Book;
use crate::bookcase::{BookCase, BookInfo};
use crate::utilities::{th_lepto_load};
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

            data.current_book
                .save(data.modified.clone(), target_path.clone());
            data.modified.clear();

            //Il currentpath diventa quello del nuovo libro
            if data.current_book.get_path() != target_path {
                let mut copy_info: BookInfo = data.get_current();
                copy_info.path = target_path.clone();
                copy_info.name = PathBuf::from(target_path.clone())
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                data.current_book.path = target_path;
                data.bookcase.library.push_back(copy_info);
                data.bookcase.update();
            }
            return Handled::Yes;
        }

        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            if data.i_mode {
                /* Qui stiamo prendendo un immagine per usare l'OCR */
                data.i_mode = false;

                th_lepto_load(ctx.get_external_handle(), file_info.path.clone())

            } else {
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
                    data.error_message = Option::Some("Impossible to open selected Epub".to_string());
                }
            }
            return Handled::Yes;
        }

        if let Some(res) = cmd.get(FINISH_SLOW_FUNCTION) {
            // If the command we received is `FINISH_SLOW_FUNCTION` handle the payload.
            if let Some((ch, off)) = res {

                data.current_book.get_mut_nav().set_ch(*ch);
                data.update_view();

                let ocr = data.get_current().ocr;
                data.view.guess_lines(ocr.get_avg_ch(), ocr.get_lines());


                data.current_book
                    .get_mut_nav()
                    .set_element_number(data.view.ocr_offset_to_element(*off));

                println!(
                    "OCR Done, ch: {}, offset di words with len()>5: {}, page element n. {}",
                    ch,
                    off,
                    data.view.ocr_offset_to_element(*off)
                );

            } else {
                data.error_message = Some("No matches were found, please try again with a better quality image.".to_string());
                data.current_book = Book::empty_book();
            }
            data.is_loading = false;
            return Handled::Yes;
        }

        if let Some(str) = cmd.get(FINISH_LEPTO_LOAD) {
            match str {
                Some(str) => {
                    match data.get_mut_current().unwrap().ocr.ocr_log(str.clone(), false){
                        Ok(map_id) => {
                            data.ocr_jump(ctx.get_external_handle(), map_id).clone()
                        },
                        Err(e) => eprintln!("{}", e)
                    }
                },
                None => {
                    data.error_message = Some("Couldn't load image".to_string());
                    data.current_book = Book::empty_book();
                }
            }
            return Handled::Yes;
        }


        if let Some(..) = cmd.get(SAVE_PANEL_CANCELLED) {
            data.is_loading = false;
            return Handled::Yes;
        }

        if let Some(..) = cmd.get(OPEN_PANEL_CANCELLED) {
            data.current_book = Book::empty_book();
            data.i_mode = false;
            data.is_loading = false;
            return Handled::Yes;
        }

        Handled::No
    }
}