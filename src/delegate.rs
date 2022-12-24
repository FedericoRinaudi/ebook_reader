use std::{env, thread};
use std::io::sink;
use std::path::PathBuf;
use druid::{commands, AppDelegate, AppLauncher, Command, DelegateCtx, Env, FileDialogOptions, FileSpec, Handled, LocalizedString, Target, Widget, WindowDesc, ExtEventSink};
use druid::im::Vector;
use druid::piet::TextStorage;
use crate::algorithms::OcrAlgorithms;
use crate::app::FINISH_SLOW_FUNCTION;
use crate::ApplicationState;
use crate::book::chapter::Chapter;
use crate::bookcase::BookInfo;
use crate::utilities::xml_to_text;
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
                data.current_book.path == target_path;
                data.bookcase.library.push_back(copy_info);
                data.bookcase.update();
            }
            return Handled::Yes;
        }

        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            if data.i_mode {
                /* Qui stiamo prendendo un immagine per usare l'OCR */
                data.i_mode = false;

                th_find_it(ctx.get_external_handle(), file_info.path.clone(), data.current_book.chapters.clone())

            } else {
                /* Qui stiamo prendendo un epub da aggiungere (?) TODO:IMPLEMENT THIS */
                println!("epub mode!")
            }
            return Handled::Yes;
        }

        if let Some(index) = cmd.get(FINISH_SLOW_FUNCTION) {
            // If the command we received is `FINISH_SLOW_FUNCTION` handle the payload.
            data.current_book.get_mut_nav().set_ch(index.clone());
            data.update_view();
            println!("OCR Done");
            data.is_loading = false;
            return Handled::Yes
        }


        Handled::No
    }
}

fn th_find_it(sink: ExtEventSink, path:PathBuf, chs:Vector<Chapter>){



    thread::spawn(move || {
        let mut lt = leptess::LepTess::new(None, "ita").unwrap();
        lt.set_image(path).unwrap();
        let text = String::from(lt.get_utf8_text().unwrap().replace("-\n", "")).replace("\n", " ").replace(".", " ");
        let index = find_it(text, chs);
        // Once the slow function is done we can use the event sink (the external handle).
        // This sends the `FINISH_SLOW_FUNCTION` command to the main thread and attach
        // the number as payload.
        sink.submit_command(FINISH_SLOW_FUNCTION, index, Target::Auto)
            .expect("command failed to submit");
    });

}



fn find_it(text:String, chs:Vector<Chapter>)->usize{
    for (index, ch) in chs.iter().enumerate(){
        let plain_text = xml_to_text(&ch.xml).replace("\n", " ").replace(".", " ");
        let p_clone = plain_text.clone();
        let t_clone = text.clone();
        if OcrAlgorithms::fuzzy_match(p_clone, t_clone, OcrAlgorithms::fuzzy_linear_compare) {
            return index;
        }
    }
    0 //TODO: Use option instead of returning default 0
}

/* PROVE OCR

// let plain_text = xml_to_text(&book.chapters[8].xml).replace("\n", " ");



   }

 */