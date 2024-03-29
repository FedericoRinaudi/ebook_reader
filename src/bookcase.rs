use crate::ocr::{OcrData, SerializableOcrData};
use druid::{im::Vector, Data, ImageBuf, Lens};
use epub::doc::EpubDoc;
use isolang::Language;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use std::{env, fs};

const FILE_NAME: &str = "meta.json";
//const FILE_NAME: &str = "meta.bin";

#[derive(Default, Clone, Data, Lens, Debug)]
pub struct BookInfo {
    pub name: String,
    pub path: String,
    pub start_chapter: usize,
    pub start_element_number: usize,
    pub cover_path: String,
    pub cover_buf: ImageBuf,
    pub ocr: OcrData,
    pub mapped_pages: Vector<usize>,
    pub title: String,
    pub description: String,
    pub language: String,
    pub creator: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct SerializableBookInfo {
    pub name: String,
    pub path: String,
    pub start_chapter: usize,
    pub start_element_number: usize,
    pub cover_path: String,
    pub ocr: SerializableOcrData,
    pub mapped_pages: Vec<usize>,
    pub title: String,
    pub description: String,
    pub language: String,
    pub creator: String,
}

impl From<BookInfo> for SerializableBookInfo {
    fn from(b: BookInfo) -> Self {
        SerializableBookInfo {
            name: b.name,
            path: b.path,
            start_chapter: b.start_chapter,
            start_element_number: b.start_element_number,
            cover_path: b.cover_path,
            ocr: b.ocr.into(),
            mapped_pages: b.mapped_pages.iter().map(|m| *m).collect(),
            title: b.title,
            description: b.description,
            language: b.language,
            creator: b.creator,
        }
    }
}

impl From<SerializableBookInfo> for BookInfo {
    fn from(b: SerializableBookInfo) -> Self {
        BookInfo {
            name: b.name,
            path: b.path,
            start_chapter: b.start_chapter,
            start_element_number: b.start_element_number,
            cover_path: b.cover_path.clone(),
            cover_buf: ImageBuf::from_file(b.cover_path)
                .unwrap_or(ImageBuf::from_file("./images/default.jpg").unwrap()),
            ocr: b.ocr.into(),
            mapped_pages: b.mapped_pages.iter().map(|m| *m).collect(),
            title: b.title,
            description: b.description,
            language: b.language,
            creator: b.creator,
        }
    }
}

impl BookInfo {
    pub fn new(path: String) -> Result<Self, String> {
        let mut doc = match EpubDoc::new(&path) {
            Ok(d) => d,
            Err(_) => return Err(String::new()),
        };
        let cover_path = Self::get_image(&mut doc);
        let title = doc
            .mdata("title")
            .unwrap_or("unknown".to_string())
            .replace("|", "_");
        let creator = doc
            .mdata("creator")
            .unwrap_or("unknown".to_string())
            .replace("|", "_");
        let language = Language::from_639_1(
            &doc.mdata("language")
                .unwrap_or("en".to_string())
                .split("-")
                .next()
                .unwrap_or("en")
                .to_string(),
        )
        .unwrap_or(Language::from_639_1("en").unwrap())
        .to_639_3()
        .to_string();
        let description = doc.mdata("description").unwrap_or("".to_string());

        let name = PathBuf::from(path.clone())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        Ok(Self {
            name,
            path,
            start_chapter: 0,
            start_element_number: 0,
            cover_path: cover_path.clone(),
            cover_buf: ImageBuf::from_file(cover_path)
                .unwrap_or(ImageBuf::from_file("./images/default.jpg").unwrap()),
            ocr: OcrData::new(),
            mapped_pages: Vector::new(),
            title,
            description,
            language,
            creator,
        })
    }

    pub fn get_path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }

    fn get_image(doc: &mut EpubDoc<BufReader<File>>) -> String {
        let title = doc.mdata("title").unwrap().replace("|", "_");

        let cover_data = match doc.get_cover() {
            Ok(data) => data,
            Err(_) => return String::from("./images/default.jpeg"),
        };
        let path = String::from("./images/") + title.as_str() + ".jpeg";
        File::create(path.clone())
            .unwrap()
            .write_all(&cover_data)
            .expect("Couldn't create a cover image");
        path
    }
}

#[derive(Default, Clone, Data, Lens)]
pub struct BookCase {
    pub(crate) library: Vector<BookInfo>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SerializableBookCase {
    pub(crate) library: Vec<SerializableBookInfo>,
}

impl From<BookCase> for SerializableBookCase {
    fn from(b: BookCase) -> Self {
        SerializableBookCase {
            library: b.library.iter().map(|el| el.clone().into()).collect(),
        }
    }
}

impl From<SerializableBookCase> for BookCase {
    fn from(b: SerializableBookCase) -> Self {
        BookCase {
            library: b.library.iter().map(|el| (*el).clone().into()).collect(),
        }
    }
}

impl BookCase {
    pub fn new() -> Self {
        /*
        Constructor:
         1. Read books in folder into folder_books : Vec<String>
         2. Read from meta file into saved_books : HashMap<String, BookInfo>
         3. Create a Vec<BookInfo> based on folder books taking info present in saved books if present
         4. Update saved_books with the new vector
        */

        let mut instance = BookCase {
            library: Vector::new(),
        };
        /*
        let mut folder_books: Vec<String> = Vec::new(); //contiene i libri letti in WalkDir
        for entry in WalkDir::new("./libri/").into_iter().skip(1) {
            folder_books.push((*(entry.unwrap().path().to_str().unwrap())).to_string());
        }
        */

        let mut saved_books: HashMap<String, BookInfo> = Self::fetch_saved(); //contiene tutti i libri letti dal file
        if instance.populate(&mut saved_books) {
            instance.update_meta()
        }
        instance
    }

    fn fetch_saved() -> HashMap<String, BookInfo> {
        let mut library: HashMap<String, BookInfo> = HashMap::new();
        match File::open(FILE_NAME) {
            Ok(mut file) => {
                let mut buf = String::new();
                let cwd = env::current_dir().unwrap();
                file.read_to_string(&mut buf).unwrap();
                let ser_l: SerializableBookCase = match serde_json::from_str(&buf) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("{}", e);
                        panic!();
                    }
                };
                let l: BookCase = ser_l.into();
                for book_info in l.library {
                    let absolute_path = PathBuf::from(book_info.path.clone());
                    let relative_path = match absolute_path.clone().strip_prefix(cwd.clone()) {
                        Ok(path) => ".".to_string() + path.to_str().unwrap(),
                        Err(_e) => {
                            absolute_path.clone().to_str().unwrap().to_string()
                        }
                    };
                    library
                        .entry(relative_path) /* In caso di duplicati */
                        .or_insert(book_info);
                }
                library
            }
            Err(_) => {
                eprintln!("No meta file found");
                return library;
            }
        }
    }

    fn populate(&mut self, saved_books: &mut HashMap<String, BookInfo>) -> bool {
        let mut file_need_update = false;
        /* Aggiungiamo libri al di fuori della cartella libri */

        for fs_book in saved_books.into_iter() {
            match fs::metadata(fs_book.1.path.clone()) {
                Ok(_) => self.library.push_back(fs_book.1.clone()),
                Err(_) => {
                    file_need_update = true;
                    println!("File not found at path {}", fs_book.1.path.clone())
                }
            }
        }

        file_need_update
    }

    pub fn update_meta(&self) {
        /* Write file containing our BookInfos */
        fs::write(FILE_NAME, "").expect("Failed to write to output.txt");
        let mut output = match OpenOptions::new().write(true).open(FILE_NAME) {
            Ok(out) => out,
            Err(_) => OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(FILE_NAME)
                .unwrap(),
        };
        let ser_book_case: SerializableBookCase = self.clone().into();
        serde_json::to_writer(&mut output, &ser_book_case).unwrap();
    }
}
