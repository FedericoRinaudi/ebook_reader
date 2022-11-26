pub mod chapter;
mod epub_text;
pub mod page;
pub(crate) mod page_element;
use walkdir::{WalkDir, DirEntry};

use std::{fs, io};
use std::env::current_dir;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use crate::book::chapter::Chapter;
use crate::book::page::Page;
use druid::{im::Vector, Data, Lens};
use epub::doc::EpubDoc;
use std::option::Option::{None, Some};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use druid::platform_menus::mac::file::print;
use zip::write::FileOptions;

#[derive(Default, Clone, Data, Lens)]
pub struct Book {
    pub chapters_xml_and_path: Vector<(String, String)>, //TODO: faccio una struct anzi che tuple
    pub path: String,
    pub current_chapter_number: usize,
    current_page_number_in_chapter: usize,
    current_page_number: usize,
    pub current_chapter: Chapter,
    pub current_page: Page,
    // pub current_html: Page,
    pub edit: bool //Make enum eventually
}

impl Book {

    pub fn empty_book() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.chapters_xml_and_path.len() == 0
    }

    pub fn new<P: AsRef<Path>>(
        path: P,
        initial_chapter_number: usize,
        initial_page_number_in_chapter: usize,
        //   initial_page_number:usize,
    ) -> Result<Self, ()> {
        let book_path = path.as_ref().to_path_buf();
        let mut epub_doc = match EpubDoc::new(path) {
            Ok(epub) => epub,
            Err(_) => return Result::Err(()),
        };
        let mut chapters_xml_and_path = Vector::new();
        while {
            //TODO: gestisco diversamente l'unwrap... qua in effetti spesso va in errore
            let chapter_xml = epub_doc.get_current_str().unwrap();
            //TODO: faccio una funzione
            let chapter_path = epub_doc.get_current_path().unwrap();
            chapters_xml_and_path.push_back((
                chapter_xml,
                chapter_path.into_os_string().into_string().unwrap(),
            ));
            epub_doc.go_next().is_ok()
        } {}

        let initial_chapter = match chapters_xml_and_path.get(initial_chapter_number) {
            Some((chapter_xml, chapter_path)) => Chapter::new(
                &chapter_path,
                &book_path.clone().into_os_string().into_string().unwrap(),
                chapter_xml.clone(),
            ),
            None => return Err(()),
        };

        let initial_page = match initial_chapter.get_page(initial_page_number_in_chapter) {
            Some(page) => page,
            None => return Err(()),
        };

        Result::Ok(
            //TODO: gestisco diversamente gli unwrap (se per esempio avessi il primo capitolo vuoto si spaccherebbe tutto, è corretto?)
            Self {
                chapters_xml_and_path,
                path: book_path.into_os_string().into_string().unwrap(),
                current_chapter_number: initial_chapter_number,
                current_page_number_in_chapter: initial_page_number_in_chapter,
                //  current_page_number:initial_page_number,
                current_page_number: initial_page_number_in_chapter,
                current_chapter: initial_chapter,
                current_page: initial_page,
                edit: false
            },
        )
    }

    /*
    Save new xml to a new version of the archive
    */
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
                let (chapter_xml, chapter_path) = self
                    .chapters_xml_and_path
                    .get((*self).current_chapter_number)
                    .unwrap()
                    .clone();
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
}
