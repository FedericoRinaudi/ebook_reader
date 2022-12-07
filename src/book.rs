pub mod chapter;
mod epub_text;
pub mod page;
pub(crate) mod page_element;
use walkdir::WalkDir;

use crate::book::chapter::Chapter;
use crate::book::page::Page;
use crate::book::page_element::PageElement;
use druid::{im::Vector, Data, Lens, ImageBuf, FontStyle, FontWeight};
use epub::doc::EpubDoc;
use std::env::current_dir;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::option::Option::{None, Some};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use druid::im::HashMap;
use druid::text::Attribute;
use zip::write::FileOptions;
use crate::book::epub_text::AttributeCase;

#[derive(Default, Clone, Data, Lens)]
pub struct Navigation {
    ch: usize,   // N° Capitolo corrente
    line: usize, // Pagine rimosse -> Offset nel capitolo !!!! Tipo diverso da usize(?)
}
impl Navigation {
    pub fn new(ch: usize, line: Option<usize>) -> Self {
        Navigation {
            ch,
            line: line.unwrap_or(0),
        }
    }

    pub fn get_ch(&self) -> usize {
        self.ch
    }
    pub fn set_ch(&mut self, n: usize) {
        (*self).ch = n
    }
    //pub fn get_nav(&self) -> Navigation { return self.nav.clone(); }
}

#[derive(Default, Clone, Data, Lens)]
pub struct Book {
    // -------------------- > pub chapters_xml_and_path: Vector<(String, String)>,
    // Nella Book:new dobbiamo inizializzare i vari chapters
    nav: Navigation,
    path: String, // Nel file system
    pub chapters: Vector<Chapter>,
}

impl Book {
    pub fn empty_book() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.chapters.len() == 0
    }

    pub fn new<P: AsRef<Path>>(
        path: P,
        init_chapter: usize,
        init_page: Option<usize>,
    ) -> Result<Self, ()> {
        // Apriamo come EpubDoc il file passato
        let book_path = path.as_ref().to_path_buf().into_os_string().into_string().unwrap();
        let mut epub_doc = match EpubDoc::new(path) {
            Ok(epub) => epub,
            Err(_) => return Result::Err(()),
        };

        let mut ch_vec = Vector::new();
        while {
            //La libreria che fa il parsing fallisce quando incontra &nbsp; quindi lo sostiusco a priori con uno spazio
            let ch_xml = epub_doc.get_current_str().unwrap(); //TODO: match for errors
            let ch_path = epub_doc
                .get_current_path()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap();

            //Creiamo un nuovo capitolo
            let ch = Chapter::new(ch_path, ch_xml, &book_path);

            ch_vec.push_back(ch);
            epub_doc.go_next().is_ok()
        } {}

        let nav_new = Navigation::new(init_chapter, init_page);

        Result::Ok(Self {
            path: book_path,
            nav: nav_new,
            chapters: ch_vec
        })
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn get_nav(&self) -> Navigation {
        self.nav.clone()
    }

    pub fn format_current_chapter(&self) -> Vector<PageElement> {
        (*self).chapters[self.nav.get_ch()].format(&(*self.path))
    }

    pub fn go_on(&mut self, n: usize) {
        self.nav.set_ch(
            if (self.nav.get_ch() + n) >= self.chapters.len() {
                self.chapters.len() - 1
            } else {
                self.nav.get_ch() + n
            }
        )
    }

    pub fn go_back(&mut self, n: usize) {
        self.nav.set_ch(
            if self.nav.get_ch() > n {
                self.nav.get_ch() - n
            } else {
                0
            }
        )
    }

}

    /*
    Save new xml to a new version of the archive
    */
    /*
    pub fn save_n_update(&mut self){
        /*
        Get the ZipArchive from the original file
         */
        let file = fs::File::open(&self.path.clone()).unwrap();
        file.set_permissions(fs::Permissions::from_mode(0o644)).expect("Error changing perms");
        let mut archive = zip::ZipArchive::new(file).unwrap();

        /*
        Unzip the file into a tmp dir to edit before zipping again
         */
        let mut dir = current_dir().unwrap().to_str().unwrap().to_string();
        dir.push_str("/tmp/");
        let path_dir = PathBuf::from(&dir).into_os_string();
        // println!("{:?}", path_dir);
        match archive.extract(path_dir){
            Ok(_) => (),
            Err(e) => eprintln!("{}", e)
        };

        /*
        Modify the file at path chapters_xml_and_path[current_chapter_number].1
         */
        let mut target_path = dir.clone();
        target_path.push_str(&self.chapters_xml_and_path[self.current_chapter_number].1);
        // println!("{}", dir);
        let mut target = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(target_path)
            .unwrap();
        target.write_all(&self.chapters_xml_and_path[self.current_chapter_number].0.as_bytes()).expect("Unable to write data");

        /*
        Change the old epub file.epub -> file-old.epub
        Zip the file again with the original epub's name
        Cleanup by deleting the tmp folder
         */

        let walkdir = WalkDir::new(dir.to_string());
        let it = walkdir.into_iter();
        let newpath = self.path.clone().replace(".epub","-old.epub");
        fs::rename(&self.path, newpath).unwrap();
        let writer = File::create(PathBuf::from(&self.path)).unwrap();

        let mut zip = zip::ZipWriter::new(writer);
        let options = FileOptions::default()
            //.compression_method(method)
            .unix_permissions(0o755);

        let mut buffer = Vec::new();
        for entry in it.filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = path.strip_prefix(Path::new(&dir)).unwrap();
            if path.is_file() {
                zip.start_file_from_path(name, options).unwrap();
                let mut f = File::open(path).unwrap();
                f.read_to_end(&mut buffer).unwrap();
                zip.write_all(&*buffer).unwrap();
                buffer.clear();
            } else if name.as_os_str().len() != 0 {
                zip.add_directory_from_path(name, options).unwrap();
            }
        }
        zip.finish().unwrap();
        fs::remove_dir_all(&dir).unwrap();

        /*
        Update the current model so that changes show without having to update the book
         */
        let (chapter_xml, chapter_path) = self
            .chapters_xml_and_path
            .get((*self).current_chapter_number)
            .unwrap()
            .clone();
        (*self).current_chapter = Chapter::new(&chapter_path, &self.path, chapter_xml);
        (*self).current_page_number_in_chapter = self.current_page_number_in_chapter;
        (*self).current_page = self
            .current_chapter
            .get_page((*self).current_page_number_in_chapter)
            .unwrap();
    }
    */
    /*

    pub fn go_to_next_page_if_exist(&mut self) {
        if (*self).current_page_number_in_chapter + 1 >= self.current_chapter.get_number_of_pages()
        {
            //SONO ALL'ULTIMA PAGINA DEL CAPITOLO
            if (*self)
                .chapters_xml_and_path
                .get((*self).current_chapter_number + 1)
                .is_some()
            {
                //NON SONO ALL'ULTIMO CAPITOLO?
                (*self).current_chapter_number += 1;
                (*self).current_page_number_in_chapter = 0;
                (*self).current_page_number += 1;
                let (chapter_xml, chapter_path) = self
                    .chapters_xml_and_path
                    .get((*self).current_chapter_number)
                    .unwrap()
                    .clone();
                (*self).current_chapter = Chapter::new(&chapter_path, &self.path, chapter_xml);
            } else {
                return;
            };
        } else {
            (*self).current_page_number_in_chapter += 1;
            (*self).current_page_number = (*self).current_page_number + 1;
        }
        (*self).current_page = self
            .current_chapter
            .get_page((*self).current_page_number_in_chapter)
            .unwrap();
    }
    pub fn go_fast_forward_if_exist(&mut self) {
        for _ in 0..10 {
            if (*self).current_page_number_in_chapter + 1
                >= self.current_chapter.get_number_of_pages()
            {
                //SONO ALL'ULTIMA PAGINA DEL CAPITOLO
                if (*self)
                    .chapters_xml_and_path
                    .get((*self).current_chapter_number + 1)
                    .is_some()
                {
                    //NON SONO ALL'ULTIMO CAPITOLO?
                    (*self).current_chapter_number += 1;
                    (*self).current_page_number_in_chapter = 0;
                    (*self).current_page_number += 1;
                    let (chapter_xml, chapter_path) = self
                        .chapters_xml_and_path
                        .get((*self).current_chapter_number)
                        .unwrap()
                        .clone();
                    (*self).current_chapter = Chapter::new(&chapter_path, &self.path, chapter_xml);
                } else {
                    return;
                };
            } else {
                (*self).current_page_number_in_chapter += 1;
                (*self).current_page_number = (*self).current_page_number + 1;
            }
            (*self).current_page = self
                .current_chapter
                .get_page((*self).current_page_number_in_chapter)
                .unwrap();
        }
    }

    pub fn go_to_prev_page_if_exist(&mut self) {
        if (*self).current_page_number_in_chapter == 0 {
            //SONO ALLA PRIMA PAGINA DEL CAPITOLO, TORNO ALL'UlTIMA PAGINA DEL PRECEDENTE
            if (*self).current_chapter_number > 0 {
                (*self).current_chapter_number -= 1;
                println!("{}", self.current_chapter_number);
                let (chapter_xml, chapter_path) = self
                    .chapters_xml_and_path
                    .get((*self).current_chapter_number)
                    .expect("Errore")
                    .clone();
                println!("Sono arrivato al chap {} pag {}", self.current_chapter_number, self.current_page_number);
                (*self).current_chapter = Chapter::new(&chapter_path, &self.path, chapter_xml);
                (*self).current_page_number_in_chapter = self.current_chapter.get_number_of_pages();
            } else {
                return;
            }
        }
        (*self).current_page_number_in_chapter = (*self).current_page_number_in_chapter - 1;
        (*self).current_page_number = (*self).current_page_number - 1;
        (*self).current_page = self
            .current_chapter
            .get_page((*self).current_page_number_in_chapter)
            .unwrap();
    }

    pub fn go_fast_back_if_exist(&mut self) {
        for _ in 0..10 {
            if (*self).current_page_number_in_chapter == 0 {
                //SONO ALLA PRIMA PAGINA DEL CAPITOLO, TORNO ALL'UlTIMA PAGINA DEL PRECEDENTE
                if (*self).current_chapter_number > 0 {
                    (*self).current_chapter_number -= 1;
                    let (chapter_xml, chapter_path) = self
                        .chapters_xml_and_path
                        .get((*self).current_chapter_number)
                        .unwrap()
                        .clone();
                    (*self).current_chapter = Chapter::new(&chapter_path, &self.path, chapter_xml);
                    (*self).current_page_number_in_chapter =
                        self.current_chapter.get_number_of_pages();
                } else {
                    return;
                }
            }
            (*self).current_page_number_in_chapter = (*self).current_page_number_in_chapter - 1;
            (*self).current_page_number = (*self).current_page_number - 1;
            (*self).current_page = self
                .current_chapter
                .get_page((*self).current_page_number_in_chapter)
                .unwrap();
        }
    }

    pub fn get_current_page_number(&self) -> usize {
        return (*self).current_page_number;
    }
    */

    /*

    pub(crate) fn get_image(&self, book_path: String) -> String
    {
        let doc = EpubDoc::new(book_path);
        assert!(doc.is_ok());
        let mut doc = doc.unwrap();
        let title = doc.mdata("title").unwrap().replace(" ", "_").split('/').into_iter().next().unwrap().to_string();

        let cover_data = doc.get_cover().unwrap();

        let mut path = String::from("./images/");
        path.push_str(title.as_str());
        path.push_str(".jpeg");

        let f = fs::File::create(path.clone());
        assert!(f.is_ok());
        let mut f = f.unwrap();
        let _ = f.write_all(&cover_data);

        return path;
    }
    */
