mod app;
mod book;
mod bookcase;
mod controllers;
mod utilities;
mod view;

use druid::{AppLauncher, WindowDesc};
use view::view::WINDOW_TITLE;

use crate::app::ApplicationState;
use crate::book::page_element::PageElement;
use crate::book::Book;
use crate::view::render::build_main_view;

/* FORSE INUTILE
struct TakeFocus;

impl<T, W: Widget<T>> Controller<T, W> for TakeFocus {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::WindowConnected = event {
            ctx.request_focus();
        }
        child.event(ctx, event, data, env)
    }
}*/

fn main() {
    //TODO:Usa struttura diversa, hashmap?!

    //let book = Book::new("./libri/saviano.epub", 0, Option::None).unwrap();
    let app = ApplicationState::new();

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
        //TODO: Refactor function
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
        //TODO: Possibly use a Vec<String> to handle line reading
        let mut word = line.as_ref().unwrap().split_whitespace().into_iter(); //TODO:Separator becomes | from whitespace
        //TODO: E' presente nall hashmap? se si creo e pusho bookinfo e rimuovo dalla map
        library.push_back(BookInfo {
            //name:word.next().unwrap().to_string().clone(),
            name: word.next().unwrap().to_string().clone(),
            start_chapter: usize::from_str_radix(word.next().unwrap(), 10).unwrap(),
            //start_page_in_chapter: usize::from_str_radix(word.next().unwrap(), 10).unwrap(),
            //tot_pages: usize::from_str_radix(word.next().unwrap(), 10).unwrap(),
            image: word.next().unwrap().to_string().clone(),
        })
    }
    //TODO: A fine iter devo pushare gli elementi che ho ancora in folder book
    //TODO: Scriviamo tutto book info per risolvere conflitti
    library
}
*/
