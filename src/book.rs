mod epub_text;
pub mod page;
mod chapter;
pub(crate) mod page_element;

use druid::{Data, im::Vector, Lens};
use std::option::Option::{None, Some};
use std::path::Path;
use epub::doc::EpubDoc;
use crate::book::page::Page;
use crate::book::chapter::Chapter;

#[derive(Clone, Data, Lens)]
pub struct Book {
    chapters_xml_and_path: Vector<(String, String)>, //TODO: faccio una struct anzi che tuple
    path: String,
    current_chapter_number: usize,
    current_page_number_in_chapter: usize,
    current_chapter: Chapter,
    pub current_page: Page
}


impl Book {

    /*pub fn new<P: AsRef<Path>>(path: P, initial_chapter_number: usize, initial_page_number_in_chapter: usize) -> Result<Self, ()> {
        let book_path = path.as_ref().to_path_buf();
        let mut epub_doc = match EpubDoc::new(path){
            Ok(epub) => epub,
            Err(_) => return Result::Err(())
        };

        let mut chapters_xml_and_path = Vector::new();

        let spine = Book::get_spine(&epub_doc);
        let mut chapters_xml = Vector::new();

        for (_,path) in spine {
            let xml = epub_doc.get_resource_str_by_path(path).unwrap();
            chapters_xml.push_back(xml);
        };

        let initial_chapter =
            match chapters_xml_and_path.get(initial_chapter_number){
            Some((chapter_xml, chapter_path)) => Chapter::new(&chapter_path, &book_path.clone().into_os_string().into_string().unwrap(), chapter_xml.clone()),
            None => return Err(())
        };

        let initial_page = match initial_chapter.get_page(initial_page_number_in_chapter){
            Some(page) => page,
            None => return Err(())
        };

        Result::Ok(
            //TODO: gestisco diversamente gli unwrap (se per esempio avessi il primo capitolo vuoto si spaccherebbe tutto, è corretto?)
            Self {
                chapters_xml_and_path,
                path: book_path.into_os_string().into_string().unwrap(),
                current_chapter_number: initial_chapter_number,
                current_page_number_in_chapter: initial_page_number_in_chapter,
                current_chapter: initial_chapter,
                current_page: initial_page
            }
        )
    }

    // Alice's Fix per compatibilità Windows

    fn convert_path_separators(href: &str) -> String {
        let mut path = String::from(href);
        if cfg!(windows) {
            path = path.replace("\\", "/");
            return path
        }
        path
    }

    fn get_spine(epub_doc: &EpubDoc<File>) -> Vec<(String, String)> {

        let mut manifest_new:HashMap<&str, String> = HashMap::new();
        (&epub_doc.resources).into_iter().for_each(|e|{
            //let start = &(e.1).0.to_str().unwrap().find("@");
            manifest_new.insert(&*e.0, Book::convert_path_separators((e.1).0.to_str().unwrap())); //[start..]);
        });
        // let meta = &epub_doc.metadata;
        // println!("{}", self.meta);
        let mut spine: Vec<(String, String)> = Vec::new();
        epub_doc.spine.iter().enumerate().for_each(|e|{
            spine.push((e.0.to_string(), manifest_new.remove((e.1).as_str()).unwrap().to_string()));
        });
        //println!("{:?}", spine);
        spine
    }

    //
    */

    pub fn new<P: AsRef<Path>>(path: P, initial_chapter_number: usize, initial_page_number_in_chapter: usize) -> Result<Self, ()> {
        let book_path = path.as_ref().to_path_buf();
        let mut epub_doc = match EpubDoc::new(path){
            Ok(epub) => epub,
            Err(_) => return Result::Err(())
        };
        let mut chapters_xml_and_path = Vector::new();

        while {
            //TODO: gestisco diversamente l'unwrap... qua in effetti spesso va in errore
            let chapter_xml = epub_doc.get_current_str().unwrap();
            //TODO: faccio una funzione
            let chapter_path = epub_doc.get_current_path().unwrap();
            chapters_xml_and_path.push_back((chapter_xml, chapter_path.into_os_string().into_string().unwrap()));
            epub_doc.go_next().is_ok()
        } {}

        let initial_chapter =match chapters_xml_and_path.get(initial_chapter_number){
            Some((chapter_xml, chapter_path)) => Chapter::new(&chapter_path, &book_path.clone().into_os_string().into_string().unwrap(), chapter_xml.clone()),
            None => return Err(())
        };

        let initial_page = match initial_chapter.get_page(initial_page_number_in_chapter){
            Some(page) => page,
            None => return Err(())
        };

        Result::Ok(
            //TODO: gestisco diversamente gli unwrap (se per esempio avessi il primo capitolo vuoto si spaccherebbe tutto, è corretto?)
            Self {
                chapters_xml_and_path,
                path: book_path.into_os_string().into_string().unwrap(),
                current_chapter_number: initial_chapter_number,
                current_page_number_in_chapter: initial_page_number_in_chapter,
                current_chapter: initial_chapter,
                current_page: initial_page
            }
        )
    }

    pub fn go_to_next_page_if_exist(&mut self) {
        (*self).current_page_number_in_chapter += 1;
        if (*self).current_page_number_in_chapter >= self.current_chapter.get_number_of_pages() { //SONO ALL'ULTIMA PAGINA DEL CAPITOLO
            if (*self).chapters_xml_and_path.get((*self).current_chapter_number + 1).is_some() { //NON SONO ALL'ULTIMO CAPITOLO?
                (*self).current_chapter_number += 1;
                (*self).current_page_number_in_chapter = 0;
                let (chapter_xml, chapter_path) = self.chapters_xml_and_path.get((*self).current_chapter_number).unwrap().clone();
                (*self).current_chapter = Chapter::new(&chapter_path, &self.path, chapter_xml);
            }else{
                return;
            };
        }
        (*self).current_page = self.current_chapter.get_page((*self).current_page_number_in_chapter).unwrap();
    }

    pub fn go_to_prev_page_if_exist(&mut self){
        if (*self).current_page_number_in_chapter == 0 { //SONO ALLA PRIMA PAGINA DEL CAPITOLO, TORNO ALL'UlTIMA PAGINA DEL PRECEDENTE
            if (*self).current_chapter_number > 0 {
                (*self).current_chapter_number -= 1;
                let (chapter_xml, chapter_path) = self.chapters_xml_and_path.get((*self).current_chapter_number).unwrap().clone();
                (*self).current_chapter = Chapter::new(&chapter_path, &self.path, chapter_xml);
                (*self).current_page_number_in_chapter = self.current_chapter.get_number_of_pages();
            } else {
                return;
            }
        }
        (*self).current_page_number_in_chapter = (*self).current_page_number_in_chapter - 1;
        (*self).current_page = self.current_chapter.get_page((*self).current_page_number_in_chapter).unwrap();
    }

}
