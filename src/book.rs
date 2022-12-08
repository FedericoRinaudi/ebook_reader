pub mod chapter;
mod epub_text;
pub(crate) mod page_element;
use walkdir::WalkDir;

use crate::book::chapter::Chapter;
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
        (*self).chapters[self.nav.get_ch()].format()
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


    /*
    Save new xml to a new version of the archive
    */

    pub fn save(&self) {
        /*
        Get the ZipArchive from the original file
         */
        let file = fs::File::open(&self.path.clone()).unwrap();
        //file.set_permissions(fs::Permissions::from_mode(0o644)).expect("Error changing perms");
        let mut archive = zip::ZipArchive::new(file).unwrap();

        /*
        Unzip the file into a tmp dir to edit before zipping again
         */
        let mut dir = current_dir().unwrap().to_str().unwrap().to_string();
        dir.push_str("/tmp/");
        let path_dir = PathBuf::from(&dir).into_os_string();
        // println!("{:?}", path_dir);

        //TODO: Different thread? Possibly

        match archive.extract(path_dir) {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e)
        }; //TODO: Propaga errore a utente

        /*
        Modify the file at path chapters_xml_and_path[current_chapter_number].1
         */
        let mut target_path = dir.clone(); // current_dir/tmp
        target_path.push_str(&self.chapters[self.get_nav().get_ch()].get_path()); // current_dir/temp/pathdelcapitolodamodificare
        // println!("{}", dir);
        let mut target = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(target_path)
            .unwrap();
        target.write_all(&self.chapters[self.get_nav().get_ch()].xml.clone().as_bytes()).expect("Unable to write data");

        /*
        Change the old epub file.epub -> file-old.epub
        Zip the file again with the original epub's name
        Cleanup by deleting the tmp folder
         */

        let walkdir = WalkDir::new(dir.to_string());
        let it = walkdir.into_iter();
        let newpath = self.path.clone().replace(".epub", "-new.epub");
        // fs::rename(&self.path, newpath).unwrap(); OLD WAY
        let writer = File::create(PathBuf::from(newpath)).unwrap();

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
        fs::remove_dir_all(&dir).unwrap(); // Cancella tmp dir
    }
}

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
