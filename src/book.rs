pub mod chapter;
mod epub_text;
pub(crate) mod page_element;

use walkdir::WalkDir;

use crate::book::chapter::Chapter;
use crate::book::page_element::PageElement;
use druid::im::{HashMap, HashSet};
use druid::{im::Vector, Data, ExtEventSink, ImageBuf, Lens};
use epub::doc::EpubDoc;
use std::env::current_dir;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{fmt, fs};
use zip::write::FileOptions;

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
            let ch_xml = epub_doc.get_current_str()?; //TODO: match for errors
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
    //pub fn _get_line(&self) -> f64 {self.nav.get_line()}

    pub fn format_current_chapter(&mut self, ctx: ExtEventSink) -> Vector<PageElement> {
        (*self).chapters[self.nav.get_ch()].format(Some(&(*self).imgs), Some(ctx), &self.path)
    }

    /*pub fn _format_chapter(&mut self, chapter_n: usize) -> Vector<PageElement> {
        //TODO: Remove if unneeded
        (*self).chapters[chapter_n].format(&(*self).imgs)
    }*/

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
    #[allow(deprecated)]
    pub fn save(&mut self, set: HashSet<usize>, target_path: String) {
        let file_path = (&self).path.clone();
        let file = fs::File::open(file_path).unwrap();

        //file.set_permissions(fs::Permissions::from_mode(0o644)).expect("Error changing perms");
        let mut archive = zip::ZipArchive::new(file).unwrap();

        /*
        Unzip the file into a tmp dir to edit before zipping again
         */
        let mut dir = current_dir().unwrap().to_str().unwrap().to_string();
        dir.push_str("/tmp/");
        let path_dir = PathBuf::from(&dir).into_os_string();

        //TODO: Different thread? Possibly

        match archive.extract(path_dir) {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e),
        }; //TODO: Propaga errore a utente

        /*
        Modify the file at path chapters_xml_and_path[current_chapter_number].1
         */

        for ch_n in set {
            let mut target_path = dir.clone(); // current_dir/tmp
            target_path.push_str(&self.chapters[ch_n].get_path()); // current_dir/temp/pathdelcapitolodamodificare
                                                                   // println!("{}", dir);
                                                                   //Svuotiamo il file prima di sovrascriverlo altrimenti malfunziona
            fs::write(target_path.clone(), "").expect("Failed to write to output.txt");
            let mut target = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(target_path.clone())
                .expect(target_path.as_str());
            target
                .write_all(&self.chapters[ch_n].xml.clone().as_bytes())
                .expect("Unable to write data");
        }

        /*
        Zip the file again with the target epub's name
         */

        let walkdir = WalkDir::new(dir.to_string());
        let it = walkdir.into_iter();

        let writer = match OpenOptions::new()
            .write(true)
            .open(PathBuf::from(target_path.clone()))
        {
            Ok(out) => out,
            Err(_) => OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(PathBuf::from(target_path.clone()))
                .unwrap(),
        };

        let mut zip = zip::ZipWriter::new(writer);
        let options = FileOptions::default()
            //.compression_method(method)
            .unix_permissions(0o755); //TODO:WIndows Compatibility

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

        //Cleanup by deleting the tmp folder
        fs::remove_dir_all(&dir).unwrap(); // Cancella tmp dir
    }
}
