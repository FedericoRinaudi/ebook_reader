use druid::{im::Vector, Data, Lens};
use epub::doc::EpubDoc;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

const FILE_NAME: &str = "meta.txt";

#[derive(Default, Clone, Data, Lens, Debug)]
pub struct BookInfo {
    pub name: String,
    path: String,
    pub start_chapter: usize,
    pub start_line: f64,
    pub cover_path: String,
}

impl BookInfo {
    fn new(path: String, start_chapter: usize, start_line:f64, cover_path: String) -> Self {
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
            start_line,
            cover_path,
        }
    }

    pub fn get_path(&self)->PathBuf {
        PathBuf::from(&self.path)
    }
}

#[derive(Default, Clone, Data, Lens)] //TODO: Cleanup
pub struct BookCase {
    pub(crate) library: Vector<BookInfo>,
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

        let mut folder_books: Vec<String> = Vec::new(); //contiene i libri letti in WalkDir
        for entry in WalkDir::new("./libri/").into_iter().skip(1) {
            folder_books.push((*(entry.unwrap().path().to_str().unwrap())).to_string());
        }

        let mut saved_books: HashMap<String, BookInfo> = Self::fetch_saved(); //contiene tutti i libri letti dal file
        if instance.populate(&folder_books, &mut saved_books) {
            instance.update()
        }
        instance
    }

    fn fetch_saved() -> HashMap<String, BookInfo> {
        let mut library: HashMap<String, BookInfo> = HashMap::new();
        match File::open(FILE_NAME) {
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    let mut words: Vec<String> = line
                        .as_ref()
                        .unwrap()
                        .split('|')
                        .map(|s| s.to_string())
                        .collect();

                    if words.len() >= 4 { // TODO: Check "words" validity
                        // Example of valid words: path ch_num ch_offset img_path
                        library
                            .entry(words[0].clone()) /* In caso di duplicati */
                            .or_insert(BookInfo::new(
                                words.remove(0),
                                usize::from_str_radix(&(words.remove(0)), 10).unwrap(),
                                words.remove(0).parse().unwrap(),
                                //f64::from_str_radix(&(words.remove(0)), 10).unwrap(),
                                words.remove(0),
                            ));
                    }
                }
                library
            },
            Err(_) => {
                eprintln!("No meta file found");
                return library
            }
        }
    }

    fn populate(
        &mut self,
        folder_books: &Vec<String>,
        saved_books: &mut HashMap<String, BookInfo>,
    ) -> bool {
        let mut file_need_update = false;
        for book_path in folder_books {
            self.library.push_back(match saved_books.get(book_path) {
                Some(book_info) => {
                    let info = book_info.clone();
                    saved_books.remove(book_path);
                    info
                }
                None => {
                    file_need_update = true;
                    BookInfo::new(book_path.clone(), 0, 0.0, Self::get_image(book_path))
                }
            })
        }
        if !saved_books.is_empty() {
            file_need_update = true
        }
        file_need_update
    }

    pub fn update(&self) {
        /* Write file containing our BookInfos */
        let mut output = match OpenOptions::new()
            .write(true)
            .open(FILE_NAME)
        {
            Ok(out) => out,
            Err(_) => OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(FILE_NAME)
                .unwrap()
        };

        for infos in self.library.iter() {
            output
                .write_all(
                    (infos.path.clone()
                        + "|"
                        + &(infos.start_chapter.to_string())
                        + "|"
                        + &(infos.start_line.to_string())
                        + "|"
                        + &(infos.cover_path.to_string())
                        + "\n")
                        .as_bytes(),
                )
                .expect("write failed");
        }
    }

    fn get_image(book_path: &str) -> String {
        //TODO: gestisco il caso di errore nell'apertura del libro
        let mut doc = EpubDoc::new(book_path.to_string()).unwrap();
        let title = doc.mdata("title").unwrap().replace("|", "_");

        println!("{}", title);
        /*.split('/')
        .into_iter()
        .next()
        .unwrap()
        .to_string();*/

        //TODO: gestisco un eventuale fallimento della get cover
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
