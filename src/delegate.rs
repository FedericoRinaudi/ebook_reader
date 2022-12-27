use crate::algorithms::OcrAlgorithms;
use crate::app::FINISH_SLOW_FUNCTION;
use crate::book::chapter::Chapter;
use crate::book::Book;
use crate::bookcase::{BookCase, BookInfo};
use crate::utilities::xml_to_text;
use crate::ApplicationState;
use druid::commands::{OPEN_PANEL_CANCELLED, SAVE_PANEL_CANCELLED};
use druid::im::Vector;
use druid::{commands, AppDelegate, Command, DelegateCtx, Env, ExtEventSink, Handled, Target};
use epub::doc::EpubDoc;
use std::path::PathBuf;
use std::{env, fs, thread};
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
                th_find_it(
                    ctx.get_external_handle(),
                    file_info.path.clone(),
                    data.current_book.chapters.clone(),
                )
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

fn th_find_it(sink: ExtEventSink, path: PathBuf, chs: Vector<Chapter>) {
    thread::spawn(move || {
        let mut lt = leptess::LepTess::new(None, "ita").unwrap();
        lt.set_image(path).unwrap();
        let text = String::from(lt.get_utf8_text().unwrap().replace("-\n", ""))
            .replace("\n", " ")
            .replace(".", " ");
        if let Some((index, offset)) = find_it(text, chs) {
            sink.submit_command(
                FINISH_SLOW_FUNCTION,
                Option::Some((index, offset)),
                Target::Auto,
            )
            .expect("command failed to submit");
        } else {
            sink.submit_command(FINISH_SLOW_FUNCTION, Option::None, Target::Auto)
                .expect("command failed to submit");
        }
    });
}

fn find_it(text: String, chs: Vector<Chapter>) -> Option<(usize, usize)> {
    for (index, ch) in chs.iter().enumerate() {
        let plain_text = xml_to_text(&ch.xml).replace("\n", " ").replace(".", " ");
        let p_clone = plain_text.clone();
        let t_clone = text.clone();
        if let Some(offset) =
            OcrAlgorithms::fuzzy_match(p_clone, t_clone, OcrAlgorithms::fuzzy_linear_compare)
        {
            return Some((index, offset));
        }
    }
    None
}

/* PROVE OCR

fn th_find_it(sink: ExtEventSink, path:PathBuf, chs:Vector<Chapter>){

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get()) // specify the number of threads in the pool
        .build()
        .unwrap();

    let mut lt = leptess::LepTess::new(None, "ita").unwrap();
    lt.set_image(path).unwrap();
    let text = String::from(lt.get_utf8_text().unwrap().replace("-\n", "")).replace("\n", " ").replace(".", " ");


    for (index, ch) in chs.into_iter().enumerate() {
        // use the thread pool to execute a task in parallel
        let sink_clone = sink.clone();
        let text_clone = text.clone();
        pool.spawn(move || {
            if let Some(_) =  find_it(text_clone, ch.clone()){
                sink_clone.submit_command(FINISH_SLOW_FUNCTION, index, Target::Auto)
                    .expect("command failed to submit")
            }
        })
    }
}



fn find_it(text:String, ch:Chapter)->Option<()>{

        let plain_text = xml_to_text(&ch.xml).replace("\n", " ").replace(".", " ");
        let p_clone = plain_text.clone();
        let t_clone = text.clone();
        if OcrAlgorithms::fuzzy_match(p_clone, t_clone, OcrAlgorithms::fuzzy_linear_compare) {
            return Some(());
        }
        None
}


 */
