use std::env;
use druid::{
    commands, AppDelegate, AppLauncher, Command, DelegateCtx, Env, FileDialogOptions, FileSpec,
    Handled, LocalizedString, Target, Widget, WindowDesc,
};
use druid::piet::TextStorage;
use crate::ApplicationState;
use crate::bookcase::BookInfo;


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
            let target_path = match absolute_path.clone().strip_prefix(cwd.clone()){
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
                data.current_book.path == target_path;
                data.bookcase.library.push_back(copy_info);
                data.bookcase.update();
            }
            return Handled::Yes;
        }
        else { Handled::No }
    }
}
        /*
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            println!("{:?}", file_info);
            return Handled::Yes;




            match std::fs::read_to_string(file_info.path()) {
                Ok(s) => {
                    let first_line = s.lines().next().unwrap_or("");
                    data.buffer = first_line.to_owned();
                }
                Err(e) => {
                    println!("Error opening file: {}", e);
                }
            }
            return Handled::Yes;
        }
        Handled::No
    }*/