pub mod chapter;
mod epub_text;
pub(crate) mod page_element;

use crate::book::chapter::Chapter;
use crate::book::page_element::PageElement;
use druid::im::{HashMap, HashSet};
use druid::{im::Vector, Data, ExtEventSink, ImageBuf, Lens};
use epub::doc::EpubDoc;
use std::env::current_dir;
use std::error::Error;
use std::fs::{OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fmt, fs};
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Default, Debug, Clone, Data, Lens)]
pub struct Navigation {
    ch: usize,             // N° Capitolo corrente
    element_number: usize, // Pagine rimosse -> Offset nel capitolo !!!! Tipo diverso da usize(?)
}
impl Navigation {
    pub fn new(ch: usize, line: usize) -> Self {
        Navigation {
            ch,
            element_number: line,
        }
    }

    pub fn get_ch(&self) -> usize {
        self.ch
    }
    pub fn set_ch(&mut self, n: usize) {
        (*self).ch = n
    }
    pub fn get_element_numer(&self) -> usize {
        self.element_number
    }
    pub fn set_element_number(&mut self, n: usize) {
        (*self).element_number = n
    }
}

#[derive(Default, Debug, Clone, Data, Lens)]
pub struct Book {
    // Nella Book:new dobbiamo inizializzare i vari chapters
    nav: Navigation,
    pub path: String, // Nel file system
    pub chapters: Vector<Chapter>,
    pub imgs: HashMap<String, ImageBuf>,
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
        init_element_number: usize,
        page_chapter: &Vector<usize>,
    ) -> Result<Self, Box<dyn Error>> {
        // Apriamo come EpubDoc il file passato
        let book_path = path
            .as_ref()
            .to_path_buf()
            .into_os_string()
            .into_string()
            .map_err(|_e| fmt::Error::default())?;

        let mut epub_doc = EpubDoc::new(path)?;

        let mut ch_vec = Vector::new();
        let mut id = 0;
        while {
            //La libreria che fa il parsing fallisce quando incontra &nbsp; quindi lo sostiusco a priori con uno spazio
            let ch_xml = epub_doc.get_current_str()?;
            let ch_path = epub_doc
                .get_current_path()?
                .into_os_string()
                .into_string()
                .map_err(|_e| fmt::Error::default())?;

            //Creiamo un nuovo capitolo
            let starting_page = match page_chapter.get(id) {
                Some(page) => *page,
                None => 0,
            };

            let ch = Chapter::new(ch_path, ch_xml, starting_page);

            ch_vec.push_back(ch);
            id += 1;
            epub_doc.go_next().is_ok()
        } {}

        let nav_new = Navigation::new(init_chapter, init_element_number);
        Ok(Self {
            path: book_path,
            nav: nav_new,
            chapters: ch_vec,
            imgs: HashMap::new(),
        })
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn get_nav(&self) -> Navigation {
        self.nav.clone()
    }

    pub fn get_mut_nav(&mut self) -> &mut Navigation {
        &mut (*self).nav
    }

    pub fn get_ch(&self) -> usize {
        self.nav.get_ch()
    }

    pub fn format_current_chapter(&mut self, ctx: ExtEventSink) -> Vector<PageElement> {
        (*self).chapters[self.nav.get_ch()].format(Some(&(*self).imgs), Some(ctx), &self.path)
    }

    pub fn go_on(&mut self, n: usize) {
        self.get_mut_nav().set_element_number(0);
        self.nav
            .set_ch(if (self.nav.get_ch() + n) >= self.chapters.len() {
                self.chapters.len() - 1
            } else {
                self.nav.get_ch() + n
            })
    }

    pub fn go_back(&mut self, n: usize) {
        self.get_mut_nav().set_element_number(0);
        self.nav.set_ch(if self.nav.get_ch() > n {
            self.nav.get_ch() - n
        } else {
            0
        })
    }

    pub fn update_xml(&mut self, xml: String) {
        (*self).chapters[self.nav.get_ch()].xml = xml;
    }

    /*
    Save new xml to a new version of the archive
    */

    pub fn save(&mut self, set: HashSet<usize>, target_path: String) -> Result<(), Box<dyn Error>> {

        let file_path = (&self).path.clone();

        let new_target_path = if target_path == file_path {
            let mut new = current_dir()?.to_str().ok_or("No string conversion")?.to_string();
            new.push_str("/tmp.epub");
            new
        } else {
            let _ = fs::remove_file(&target_path); //Non mettere '?' -> se non c'è non è un problema: con questa istruzione mi assicuro soltanto che non esista già
            target_path.clone()
        };

        let file = fs::File::open(file_path.clone())?;
        let writer = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(PathBuf::from(new_target_path.clone()))?;



        let mut archive = zip::ZipArchive::new(file)?;

        // Set the options for creating a new file entry in the zip file
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let path_map:HashMap<String, usize> = set.iter().map(|el| (self.chapters[*el].get_path().clone(), *el)).collect();

        let mut zip_writer = ZipWriter::new(writer);

        // Iterate through the entries in the ZipArchive and add them to the ZipWriter, except for the entry to be deleted
        for i in 0..archive.len() {
            let zip_entry = archive.by_index(i)?;
            let file_name = zip_entry.name();

            if !path_map.contains_key(file_name) {
                if file_name.ends_with('/') {
                    // Add a directory to the zip file
                    zip_writer.add_directory(file_name, options)?;
                } else {
                    // Add a file to the zip file
                    zip_writer.start_file(file_name, options)?;
                    zip_writer.raw_copy_file(zip_entry)?;
                }
            } else {
                zip_writer.start_file(file_name, options)?;
                zip_writer.write_all(&self.chapters[*path_map.get(file_name).ok_or("No match")?].xml.clone().as_bytes())?;
            }

        }

        zip_writer.finish()?;

        if target_path != new_target_path {
            fs::remove_file(target_path.clone())?;
            fs::copy(&new_target_path, &target_path)?;
            fs::remove_file(&new_target_path)?;
        }

        Ok(())
    }


}
