use std::env;
use std::path::PathBuf;
use druid::{
    commands, AppDelegate, AppLauncher, Command, DelegateCtx, Env, FileDialogOptions, FileSpec,
    Handled, LocalizedString, Target, Widget, WindowDesc,
};
use druid::piet::TextStorage;
use crate::algorithms::OcrAlgorithms;
use crate::ApplicationState;
use crate::bookcase::BookInfo;
use crate::utilities::xml_to_text;


pub(crate) struct Delegate {}

impl AppDelegate<ApplicationState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut ApplicationState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::SAVE_FILE_AS) {
            let cwd = env::current_dir().unwrap();
            let absolute_path = file_info.path.clone();

            // Strip the prefix of the absolute path that is outside of the project folder
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
                let mut lt = leptess::LepTess::new(None, "ita").unwrap();
                lt.set_image(file_info.path.clone()).unwrap();
                let text = String::from(lt.get_utf8_text().unwrap().replace("-\n", "")).replace("\n", " ").replace(".", " ");

                for (index, ch) in data.current_book.chapters.iter().enumerate(){
                    let plain_text = xml_to_text(&ch.xml).replace("\n", " ").replace(".", " ");
                    let p_clone = plain_text.clone();
                    let t_clone = text.clone();
                    if OcrAlgorithms::fuzzy_match(p_clone, t_clone, OcrAlgorithms::fuzzy_linear_compare) {
                        data.current_book.get_mut_nav().set_ch(index);
                        data.update_view();
                        println!("OCR Done");
                        break;
                    }
                }
                data.i_mode = false;
            } else { println!("epub mode!") }
            data.is_loading = false;
            return Handled::Yes;
        }
        Handled::No
    }
}


/* PROVE OCR

// let plain_text = xml_to_text(&book.chapters[8].xml).replace("\n", " ");



   }

 */