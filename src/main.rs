mod book;
mod app;
mod controllers;
mod utilities;
mod view;

use druid::im::Vector;
use epub::doc::EpubDoc;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use druid::Event::WindowSize;
use druid::{AppLauncher, LocalizedString, WindowDesc};
use walkdir::WalkDir;
use view::view::{WINDOW_TITLE, View};

use crate::app::ApplicationState;
use crate::book::page_element::PageElement;
use crate::book::Book;
use crate::book::chapter::Chapter;
use crate::view::render::build_main_view;


/*
#[derive(Default, Clone, Data, Lens, Debug)]
pub struct BookInfo {
    name: String,
    start_chapter: usize,
    start_page_in_chapter: usize,
    tot_pages: usize,
    image: String,
}
*/

/*
struct TakeFocus;

impl<T, W: Widget<T>> Controller<T, W> for TakeFocus {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::WindowConnected = event {
            ctx.request_focus();
        }
        child.event(ctx, event, data, env)
    }
}
*/

/*
fn render_library() -> impl Widget<ApplicationState>{
    let mut col = Flex::column();
    let mut row = Flex::row();
    let lib = data.library.clone();
    let row_flex = 1.0 / ((lib.len() as f64 / 3.0) + 1.0);
    for (i, e) in lib.into_iter().enumerate() {
        //println!("{:?}", e);

        /*    let b = Button::new(e.name.clone()).on_click(move |_ctx, button_data: &mut ApplicationState, _env| {
            button_data.current_book = Book::new(PathBuf::from(e.name.clone()), e.start_chapter, e.start_page_in_chapter, e.tot_pages).unwrap();
        });
        */
        let b = ImageBuf::from_file(e.image.clone()).unwrap();
        let c = ControllerHost::new(
            Image::new(b).fix_width(300.0).fix_height(200.0),
            Click::new(move |_ctx, data: &mut ApplicationState, _env| {
                data.current_book = Book::new(
                    PathBuf::from(e.name.clone()),
                    e.start_chapter,
                    e.start_page_in_chapter,
                    e.tot_pages,
                )
                    .unwrap();
            }),
        );
        row.add_flex_child(c, FlexParams::new(row_flex, CrossAxisAlignment::Start));
        if i != 0 && (i + 1) % 3 == 0 {
            col.add_flex_child(row, FlexParams::new(0.3, CrossAxisAlignment::Center));
            row = Flex::row();
        }
    }
    col.add_child(row);
    col.scroll().vertical()
}
*/


fn main() {

    /*
    let mut vet: Vec<String> = Vec::new(); //contiene i libri letti in WalkDir
    for entry in WalkDir::new("./libri/").into_iter().skip(1) {
        vet.push((*(entry.unwrap().path().to_str().unwrap())).to_string());
    }

    let mut library: Vector<BookInfo> = read_from_file(); //contiene tutti i libri letti dal file
    remove_from_library(vet.clone());
    create_library(library, vet.clone());

    library = read_from_file();
    */

    let book = Book::new("./libri/saviano.epub", 0, Option::None).unwrap();
    let mut app = ApplicationState::new(book);

    // describe the main window
    let main_window = WindowDesc::new(build_main_view())
        .title(WINDOW_TITLE)
        .window_size(app.view.get_window_size_home());

    // start the application
    AppLauncher::with_window(main_window)
        .launch(app)
        .expect("Failed to launch application");
}

/*
fn remove_from_library(vet: Vec<String>) {
    let mut find = 0;
    let mut output = OpenOptions::new()
        .append(true)
        .create(true)
        .open("./tmp.txt")
        .expect("Unable to open file");
    let input = BufReader::new(File::open("file.txt").expect("Cannot open file.txt"));
    for line in input.lines() {
        let l = line
            .as_ref()
            .unwrap()
            .clone()
            .split_whitespace()
            .next()
            .unwrap()
            .to_string();
        for book in &vet {
            if l.clone() == *book {
                find = 1;
                break;
            }
        }
        if find == 1 {
            let _ = output.write_all((line.unwrap().clone() + "\n").as_bytes());
        }
        find = 0;
    }
    let _ = fs::remove_file("file.txt");
    let _ = fs::rename("tmp.txt", "file.txt");
}
fn create_library(lib: Vector<BookInfo>, vect: Vec<String>) {
    let mut output = OpenOptions::new()
        .append(true)
        .open("file.txt")
        .expect("Unable to open file");
    let mut find = 0;

    for path_element in vect {
        for file_element in &lib {
            if file_element.name.eq(&path_element.clone()) {
                find = 1;
            }
        }
        if find == 0 {
            let image = get_image(path_element.clone());
            output
                .write_all((path_element.clone() + " 0 0 0 " + image.as_str() + "\n").as_bytes())
                .expect("write failed");
        } else {
            find = 0;
        }
    }
}
fn read_from_file() -> Vector<BookInfo> {
    let mut library: Vector<BookInfo> = Vector::new(); //contiene tutti i libri letti dal file
    let reader = BufReader::new(File::open("file.txt").expect("Cannot open file.txt"));
    for line in reader.lines() {
        let mut word = line.as_ref().unwrap().split_whitespace().into_iter();
        library.push_back(BookInfo {
            //name:word.next().unwrap().to_string().clone(),
            name: word.next().unwrap().to_string().clone(),
            start_chapter: usize::from_str_radix(word.next().unwrap(), 10).unwrap(),
            start_page_in_chapter: usize::from_str_radix(word.next().unwrap(), 10).unwrap(),
            tot_pages: usize::from_str_radix(word.next().unwrap(), 10).unwrap(),
            image: word.next().unwrap().to_string().clone(),
        })
    }
    return library;
}

fn get_image(book_path: String) -> String {
    let doc = EpubDoc::new(book_path);
    assert!(doc.is_ok());
    let mut doc = doc.unwrap();
    //let name=doc.mdata("cover").unwrap();
    let title = doc
        .mdata("title")
        .unwrap()
        .replace(" ", "_")
        .split('/')
        .into_iter()
        .next()
        .unwrap()
        .to_string();

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