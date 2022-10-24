mod render;

use std::cell::RefCell;
use std::fs::File;
use std::path::PathBuf;
use epub::doc::EpubDoc;
use druid::{im::Vector, Widget, LocalizedString, WindowDesc, AppLauncher, Data, Lens, WidgetExt};
use std::rc::Rc;
use druid::text::{RichText};
use druid::widget::{Flex, Button, CrossAxisAlignment, List, RawLabel, LineBreaking};

use crate::render::render_chapter;




#[derive(Clone, Data, Lens)]
struct EbookState {
    chapter: Vector<Vector<RichText>>,
    page: Vector<RichText>,
    epub: Rc<RefCell<EpubDoc<File>>>,  //Da spostare (forse) in env
    current_page_number: usize
}

fn build_widget() -> impl Widget<EbookState> {
    let mut col = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
    let button_next = Button::new("next page").on_click(|_ctx, data: &mut EbookState, _env| {
        data.current_page_number += 1;
        if data.current_page_number >= data.chapter.len() { //IL CAPITOLO E' FINITO, CAMBIO CAPITOLO  (SE NON E' L'ULTIMO) E COMINCIO DALLA PRIMA PAGINA
            if data.epub.borrow_mut().go_next().is_ok(){
                data.chapter = render_chapter(data.epub.borrow_mut().get_current_str().unwrap());
                data.current_page_number = 0;
                data.page = data.chapter.get(data.current_page_number).unwrap().clone();
            }
        } else { //CAMBIO PAGINA
            data.page = data.chapter.get(data.current_page_number).unwrap().clone();
        }
    });
    let button_prev = Button::new("prev page").on_click(|_ctx, data: &mut EbookState, _env| {
        if data.current_page_number == 0 { //SONO ALLA PRIMA PAGINA DEL CAPITOLO, TORNO ALL'UlTIMA PAGINA DEL PRECEDENTE
            if data.epub.borrow_mut().go_prev().is_ok(){
                data.chapter = render_chapter(data.epub.borrow_mut().get_current_str().unwrap());
                data.current_page_number = data.chapter.len() - 1;
                data.page = data.chapter.get(data.current_page_number).unwrap().clone();
            }
        } else { //CAMBIO PAGINA
            data.current_page_number -= 1;
            data.page = data.chapter.get(data.current_page_number).unwrap().clone();
        }
    });
    let mut row:Flex<EbookState>=Flex::row();
    row.add_child(button_prev);
    row.add_child(button_next);

    col.add_child(row);
    let page = List::new(|| {
        let mut label = RawLabel::new();
        label.set_line_break_mode(LineBreaking::WordWrap);
        label
    }).lens(EbookState::page);
    col.add_child(page);
    col.scroll().vertical()
}


fn main() {


    //let mut epub = Arc::new(Mutex::new(EpubDoc::new(PathBuf::from("./sample.epub")).unwrap()));
    let epub = Rc::new(RefCell::new(EpubDoc::new(PathBuf::from("./libro.epub")).unwrap()));
    //const VERTICAL_WIDGET_SPACING: f64 = 20.0;
    //const TEXT_BOX_WIDTH: f64 = 200.0;
    const WINDOW_TITLE :LocalizedString<EbookState> = LocalizedString::new("Hello World!");
    // describe the main window
    let main_window = WindowDesc::new(build_widget())
        .title(WINDOW_TITLE)
        .window_size((800.0, 800.0));

    let chapter = render_chapter(epub.borrow_mut().get_current_str().unwrap());
    // create the initial app state
    let initial_state = EbookState {
        chapter: chapter.clone(),
        epub: epub.clone(),
        page: chapter.get(0).unwrap_or(&Vector::<RichText>::new()).clone(), //TODO: gestisco il caso in cui il capitolo non abbia pagine
        current_page_number: 0
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");

}