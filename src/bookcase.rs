use crate::ocr::{OcrData, SerializableOcrData};
use druid::{im::Vector, Data, Lens};
use epub::doc::EpubDoc;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{env, fs};
use serde::{Serialize, Deserialize};

const FILE_NAME: &str = "meta.txt";
//const FILE_NAME: &str = "meta.bin";


#[derive(Default, Clone, Data, Lens, Debug, PartialEq)]
pub struct BookInfo {
    pub name: String,
    pub path: String,
    pub start_chapter: usize,
    pub start_element_number: usize,
    pub cover_path: String,
    pub ocr: OcrData,
    pub mapped_pages: Vector<usize>,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SerializableBookInfo {
    pub name: String,
    pub path: String,
    pub start_chapter: usize,
    pub start_element_number: usize,
    pub cover_path: String,
    pub ocr: SerializableOcrData,
    pub mapped_pages: Vec<usize>,
}

impl From<BookInfo> for SerializableBookInfo {
    fn from(b: BookInfo) -> Self {
        SerializableBookInfo{
            name: b.name,
            path: b.path,
            start_chapter: b.start_chapter,
            start_element_number: b.start_element_number,
            cover_path: b.cover_path,
            ocr: b.ocr.into(),
            mapped_pages: b.mapped_pages.iter().map(|m|*m).collect()
        }
    }
}

impl From<SerializableBookInfo> for BookInfo {
    fn from(b: SerializableBookInfo) -> Self {
        BookInfo{
            name: b.name,
            path: b.path,
            start_chapter: b.start_chapter,
            start_element_number: b.start_element_number,
            cover_path: b.cover_path,
            ocr: b.ocr.into(),
            mapped_pages: b.mapped_pages.iter().map(|m|*m).collect()
        }
    }
}

impl BookInfo {
    pub fn new(path: String, start_chapter: usize, element_number: usize, cover_path: String) -> Self {
        let name = PathBuf::from(path.clone())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        Self {
            name,
            path,
            start_chapter,
            start_element_number: element_number,
            cover_path,
            ocr: OcrData::new(),
            mapped_pages: Vector::new(),
        }
    }

    pub fn get_path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
}

#[derive(Default, Clone, Data, Lens)] //TODO: Cleanup
pub struct BookCase {
    pub(crate) library: Vector<BookInfo>,
}

#[derive(Default, Clone, Serialize, Deserialize)] //TODO: Cleanup
pub struct SerializableBookCase {
    pub(crate) library: Vec<SerializableBookInfo>,
}

impl From<BookCase> for SerializableBookCase {
    fn from(b: BookCase) -> Self {
        SerializableBookCase{
            library: b.library.iter()
                .map(|el|el.clone().into())
                .collect()
        }
    }
}

impl From<SerializableBookCase> for BookCase {
    fn from(b: SerializableBookCase) -> Self {
        BookCase{
            library: b.library.iter()
                .map(|el|(*el).clone().into())
                .collect()
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
        //println!("folder books {:?}", folder_books.clone());

        let mut saved_books: HashMap<String, BookInfo> = Self::fetch_saved(); //contiene tutti i libri letti dal file
                                                                              //println!("saved books: {:?}", saved_books.clone());
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
                let ser_l :SerializableBookCase = match serde_json::from_str(&buf){
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("{}", e);
                        panic!();
                    }
                };
                let l :BookCase = ser_l.into();
                for book_info in l.library{
                    let absolute_path = PathBuf::from(book_info.path.clone());
                    let relative_path = match absolute_path.clone().strip_prefix(cwd.clone()) {
                        Ok(path) => ".".to_string() + path.to_str().unwrap(),
                        Err(_e) => {
                            //eprintln!("Error stripping prefix from path {}", e);
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

    fn populate(
        &mut self,
        saved_books: &mut HashMap<String, BookInfo>,
    ) -> bool {
        let mut file_need_update = false;
        /*
        for book_path in folder_books {
            //println!("Matching compare {:?} {}", saved_books.get(book_path), book_path.clone());
            self.library.push_back(match saved_books.get(book_path) {
                Some(book_info) => {
                    let info = book_info.clone();
                    saved_books.remove(book_path);
                    info
                }
                None => {
                    file_need_update = true;
                    BookInfo::new(book_path.clone(), 0, 0, Self::get_image(book_path))
                }
            })
        }
        */
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

    /*pub fn update(&self) {
        /* Write file containing our BookInfos */
        let mut output = match OpenOptions::new().write(true).open(FILE_NAME) {
            Ok(out) => out,
            Err(_) => OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(FILE_NAME)
                .unwrap(),
        };

        for infos in self.library.iter() {
            output
                .write_all(
                    (infos.path.clone()
                        + "|"
                        + &(infos.start_chapter.to_string())
                        + "|"
                        + &(infos.start_element_number.to_string())
                        + "|"
                        + &(infos.cover_path.to_string())
                        + "\n")
                        .as_bytes(),
                )
                .expect("write failed");
        }
    }*/

    pub fn get_image(book_path: &str) -> String {
        //TODO: gestisco il caso di errore nell'apertura del libro
        let mut doc = EpubDoc::new(book_path.to_string()).unwrap();
        let title = doc.mdata("title").unwrap().replace("|", "_");

        //println!("{}", title);
        /*.split('/')
        .into_iter()
        .next()
        .unwrap()
        .to_string();*/

        let cover_data = match doc.get_cover() {
            Ok(data) => data,
            Err(_) => return String::from("./images/default.jpeg"),
        };
        //TODO: se l'immagine non fosse jpeg rompo tutto
        //TODO: gestisco caso in cui fallisca
        let path = String::from("./images/") + title.as_str() + ".jpeg";
        File::create(path.clone())
            .unwrap()
            .write_all(&cover_data)
            .expect("Couldn't create a cover image");
        path
    }
}
