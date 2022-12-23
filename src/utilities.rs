use druid::{FileDialogOptions, FileSpec, ImageBuf};
use std::io::Read;
use std::path::PathBuf;

pub fn unify_paths(mut p1: PathBuf, p2: PathBuf) -> PathBuf {
    if !p1.is_dir() {
        p1.pop();
    }
    for el in p2.into_iter() {
        if el == ".." {
            p1.pop();
        } else {
            p1.push(el);
        }
    }
    p1
}

pub fn convert_path_separators(href: String) -> String {
    let mut path = String::from(href);
    if cfg!(windows) {
        path = path.replace("\\", "/");
        return path;
    }
    path
}

pub fn get_image_buf(
    book_path: &PathBuf,
    chapter_path: &PathBuf,
    image_path: PathBuf,
) -> Option<ImageBuf> {
    let zipfile = std::fs::File::open(book_path.clone()).unwrap();
    let mut archive = zip::ZipArchive::new(zipfile).unwrap();
    let complete_img_path = unify_paths(chapter_path.clone(), image_path.clone())
        .into_os_string()
        .into_string()
        .unwrap();
    let better_path = convert_path_separators(complete_img_path);
    let mut file = match archive.by_name(&better_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error in opening archive at {}", e);
            return None;
        }
    };
    let mut contents: Vec<u8> = vec![];
    //TODO: match, Err() => Default photo
    file.read_to_end(&mut contents).unwrap(); //
    match ImageBuf::from_data(&contents) {
        Ok(im) => Some(im),
        Err(_) => None, //TODO: default image
    }
}


pub fn save_file(name:String) -> FileDialogOptions {
    let epub = FileSpec::new("Epub file", &["epub"]);
    FileDialogOptions::new()
        .allowed_types(vec![epub])
        .default_type(epub)
        .default_name(name)
        .name_label("Target")
        .title("Select where to Save")
        .button_text("Save")
}

pub fn open_file() -> FileDialogOptions {
    let epub = FileSpec::new("Epub file", &["epub"]);
    FileDialogOptions::new()
        .allowed_types(vec![epub])
        .default_type(epub)
        .default_name("Book.epub")
        .name_label("Source")
        .title("Select an epub to Import")
        .button_text("Import")
}
