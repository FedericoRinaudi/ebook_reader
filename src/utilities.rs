use crate::app::{FINISH_BOOK_LOAD, FINISH_IMAGE_LOAD, FINISH_LEPTO_LOAD};
use crate::book::page_element::PageElement;
use crate::book::Book;
use crate::ContentType;
use druid::im::Vector;
use druid::{ExtEventSink, FileDialogOptions, FileSpec, ImageBuf, Target};
use roxmltree::{Document, Node, ParsingOptions};
use std::io::Read;
use std::path::PathBuf;
use std::thread;
use unicode_segmentation::UnicodeSegmentation;

pub fn unify_paths(mut p1: PathBuf, p2: PathBuf) -> PathBuf {
    for el in p2.into_iter() {
        if el == ".." {
            p1.pop();
        } else if el != "." {
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

pub fn get_image_buf(book_path: PathBuf, image_path: String) -> Option<ImageBuf> {
    let zipfile = std::fs::File::open(book_path).unwrap();
    let mut archive = zip::ZipArchive::new(zipfile).unwrap();
    let mut file = match archive.by_name(&image_path) {
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

pub fn save_file(name: String) -> FileDialogOptions {
    let epub = FileSpec::new("Epub file", &["epub"]);
    FileDialogOptions::new()
        .allowed_types(vec![epub])
        .default_type(epub)
        .default_name(name)
        .name_label("Target")
        .title("Select where to Save")
        .button_text("Save")
}

pub fn open_epub() -> FileDialogOptions {
    let epub = FileSpec::new("Epub file", &["epub"]);
    FileDialogOptions::new()
        .allowed_types(vec![epub])
        .default_type(epub)
        .default_name("Book.epub")
        .name_label("Source")
        .title("Select an epub to Import")
        .button_text("Import")
}

pub fn open_image() -> FileDialogOptions {
    let jpg = FileSpec::new("jpg file", &["jpg"]);
    let jpeg = FileSpec::new("JPeG file", &["jpeg"]);
    let jpg_caps = FileSpec::new("JPG file", &["JPG"]);
    FileDialogOptions::new()
        .allowed_types(vec![jpg, jpeg, jpg_caps])
        .default_type(jpg)
        .default_name("image.JPG")
        .name_label("Source")
        .title("Select an image to Import")
        .button_text("Import")
}

/* FOR OCR PURPOSES */

pub fn xml_to_text(xml: &str) -> String {
    let opt = ParsingOptions { allow_dtd: true };
    let doc = match Document::parse_with_options(xml, opt) {
        Result::Ok(doc) => doc,
        Err(_e) => {
            println!("Error");
            return " ".to_string();
        }
    };
    let node = doc.root_element().last_element_child().unwrap();
    let mut cur_text = String::new();
    xml_to_plain(node, &mut cur_text);
    cur_text
}

pub fn xml_to_plain(node: Node, current_text: &mut String) {
    /* Def Macros */
    macro_rules! recur_on_children {
        () => {
            for child in node.children() {
                xml_to_plain(child, current_text);
            }
        };
    }

    /*  Actual Transformation */

    if node.is_text() {
        let text = node.text().unwrap();
        let content: Vec<_> = text.split_ascii_whitespace().collect();
        if text.starts_with(char::is_whitespace) {
            current_text.push_str(" ");
        }
        current_text.push_str(&content.join(" "));
        if text.ends_with(char::is_whitespace) {
            current_text.push_str(" ");
        }
    }

    match node.tag_name().name() {
        "br" => {
            current_text.push_str("\n");
        }
        "h1" => {
            recur_on_children!();
            current_text.push_str("\n");
        }
        "h2" => {
            recur_on_children!();
            current_text.push_str("\n");
        }
        "h3" => {
            recur_on_children!();
            current_text.push_str("\n");
        }
        "h4" => {
            recur_on_children!();
            current_text.push_str("\n");
        }
        "h5" => {
            recur_on_children!();
            current_text.push_str("\n");
        }
        "h6" => {
            recur_on_children!();
            current_text.push_str("\n");
        }
        "blockquote" | "div" | "p" | "tr" => {
            recur_on_children!();
            current_text.push_str("\n");
        }
        "li" => {
            current_text.push_str("- ");
            recur_on_children!();
            current_text.push_str("\n");
        }
        _ => recur_on_children!(),
    }
}

pub fn is_part(vec: Vector<PageElement>) -> bool {
    /* Elementi non vuoti */
    let filtered = vec
        .iter()
        .filter(|elem| {
            if let ContentType::Text(text) = (*elem).clone().content {
                if text.text.trim().len() > 0 {
                    return true;
                }
                return false;
            }
            false
        })
        .map(|el| (*el).clone())
        .collect::<Vector<PageElement>>();
    /* Elementi non vuoti non maggiori di 80 char*/
    if filtered.clone().iter().any(|el| {
        if let ContentType::Text(text) = (*el).clone().content {
            if text.text.replace(" ", "").graphemes(true).count() > 80 {
                return true;
            }
        }
        return false;
    }) {
        return false;
    };
    /* Massimo 3 elementi */
    if filtered.len() > 0 && filtered.len() < 4 {
        return true;
    }

    false
}

pub fn th_lepto_load(sink: ExtEventSink, path: PathBuf, lang: &str) {
    let s = lang.to_string().clone();
    thread::spawn(move || lepto_load(sink, path, s));
}

fn lepto_load(sink: ExtEventSink, path: PathBuf, lang: String) {
    let mut lt = leptess::LepTess::new(None, &lang).unwrap();
    lt.set_image(path).unwrap();
    match lt.get_utf8_text() {
        Ok(text) => {
            // println!("{:?}", text);
            sink.submit_command(
                FINISH_LEPTO_LOAD,
                Option::Some(String::from(text)),
                Target::Auto,
            )
            .expect("command failed to submit")
        }
        Err(_) => {
            sink.submit_command(FINISH_LEPTO_LOAD, Option::None, Target::Auto)
                .expect("command failed to submit");
        }
    }
}

pub fn th_load_book(
    sink: ExtEventSink,
    path: PathBuf,
    init_ch: usize,
    init_el: usize,
    ch_pg: Vector<usize>,
) {
    thread::spawn(move || load_book(sink, path, init_ch, init_el, ch_pg));
}

fn load_book(
    sink: ExtEventSink,
    path: PathBuf,
    init_ch: usize,
    init_el: usize,
    ch_pg: Vector<usize>,
) {
    match Book::new(path, init_ch, init_el, &ch_pg) {
        Ok(book) => sink
            .submit_command(FINISH_BOOK_LOAD, Some(book), Target::Auto)
            .expect("command failed to submit"),
        Err(e) => {
            println!("Error in loading book: {}", e);
            sink.submit_command(FINISH_BOOK_LOAD, None, Target::Auto)
                .expect("command failed to submit");
        }
    }
}

pub fn th_load_image(sink: ExtEventSink, epub_img_path: String, epub_path: String) {
    thread::spawn(move || load_image(sink, epub_img_path, epub_path));
}

fn load_image(sink: ExtEventSink, epub_img_path: String, epub_path: String) {
    match get_image_buf(PathBuf::from(epub_path), epub_img_path.clone()) {
        Some(img) => sink
            .submit_command(FINISH_IMAGE_LOAD, (img, epub_img_path), Target::Auto)
            .expect("command failed to submit"),
        None => {
            sink.submit_command(
                FINISH_IMAGE_LOAD,
                (
                    ImageBuf::from_file("./images/default.jpg").unwrap(),
                    epub_img_path,
                ),
                Target::Auto,
            )
            .expect("command failed to submit");
        }
    }
}

