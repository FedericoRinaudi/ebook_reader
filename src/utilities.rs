use druid::{FileDialogOptions, FileSpec, ImageBuf};
use roxmltree::{Document, Node, ParsingOptions};
use std::io::Read;
use std::path::PathBuf;
use leptess::capi::TessPageIteratorLevel_RIL_TEXTLINE;
use unicode_segmentation::UnicodeSegmentation;

//CONTA LE RIGHE DA UNA FOTO CON LEPTESS; PROBABILMENTE ANDRA' IN UNA STRUCT APPOSITA
//VALUTIAMO SE VA BEME GUARDARE LA WIDTH O SE CONVIENE RICAVARE E CONTARE I CARATERI
pub fn page_num_lines(path: PathBuf) -> usize {
    let mut lt = leptess::LepTess::new(None, "ita").unwrap();
    lt.set_image(path).unwrap();
    lt.get_component_boxes(TessPageIteratorLevel_RIL_TEXTLINE,true)
        .unwrap()
        .into_iter()
        .filter(|el|(*el).as_ref().w > 70)
        .count()
}

pub fn page_num_lines_char_count(path: PathBuf) -> usize {
    let mut lt = leptess::LepTess::new(None, "ita").unwrap();
    lt.set_image(path).unwrap();
    lt.get_word_str_box_text(0).unwrap()
        .split("WordStr")
        .map(|s| {
            s.chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>()
        }).filter(|s| s.graphemes(true).count() > 3)
        .count()
}


//Mi sembra che il valore della media sia abbastanza buono, ma dobbiamo verificare
pub fn avg_graphemes_in_full_line(path: PathBuf) -> f64 {
    let mut lt = leptess::LepTess::new(None, "ita").unwrap();
    lt.set_image(path).unwrap();
    let lines = lt.get_utf8_text().unwrap()
        .split("\n")
        .filter(|s|s.graphemes(true).count() > 3)
        .map(|s|s.to_string())
        .collect::<Vec<String>>();
    println!("lines for avg graphemes ocr: {}", lines.len());
    //ALTERNATIVAMENTE ANZI CHE METTERE UN THRESHOLD CALCOLATO IN BASE ALLA MEDIA POSSO RICONOSCERE LE LINEE NON INTERE
    //COME LE LINEE CHE FINISCONO CON . ? ! ecc... E RIMUOVERLE PRIMA DI CALCOLARE LA MEDIA
    let threshold = (lines.iter().fold(0, |a, b|{a + b.graphemes(true).count() as i32}) as f64 / lines.len() as f64) * 4./5.;
    println!("threshold for full line: {}", threshold);
    let sum_count = lines
        .iter()
        .fold((0, 0), |(sum, count), value| {
            let grapheme_n = value.graphemes(true).count();
            if grapheme_n as f64 > threshold {
                (sum + grapheme_n as i32, count + 1)
            } else {
                (sum, count)
            }
        });
    (sum_count.0 as f64) / (sum_count.1 as f64)
}

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

fn xml_to_plain(node: Node, current_text: &mut String) {
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
