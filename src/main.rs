mod book;

use std::fs;
use std::fs::{ File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use druid::widget::{
    Button, CrossAxisAlignment, FillStrat, Flex, FlexParams, Image, Label, LineBreaking, List,
    RawLabel, ViewSwitcher,
};
use druid::{AppLauncher, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use std::path::PathBuf;
use walkdir::WalkDir;


use crate::book::page_element::PageElement;
use crate::book::Book;

#[derive(Default, Clone, Data, Lens)]
pub struct ApplicationState {
    pub current_book: Book,
}
#[derive(Default, Clone, Data, Lens)]
pub struct BookInfo {
    name:String,
    start_chapter:usize,
    start_page_in_chapter:usize
}
//SWITCH TRA VISUALIZZATORE ELENCO EBOOK E VISUALIZZATORE EBOOK
fn build_widget(library: Vec<BookInfo>) -> impl Widget<ApplicationState> {
    let a: ViewSwitcher<ApplicationState, bool> = ViewSwitcher::new(
        |data: &ApplicationState, _| data.current_book.is_empty(),
        |_, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            if data.current_book.is_empty() {
                return Box::new(Button::new("libro").on_click(
                    |_ctx, data: &mut ApplicationState, _env| {
                        data.current_book = Book::new(PathBuf::from("./libri/libro.epub"), 0, 0).unwrap();
                    },
                ));
            } else {
                return Box::new(render_book());
            }
        },
    );
    a
}
//FUNZIONE CHE CREA I BOTTONI E FA VISUALIZZARE TESTO E IMMAGINI
fn render_book() -> impl Widget<ApplicationState> {
    let mut wrapper = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
    let mut row_due = Flex::row();
    let button_next = Button::new(">").on_click(|_ctx, data: &mut ApplicationState, _env| {
        data.current_book.go_to_next_page_if_exist();
    });
    let button_fast_forward =
        Button::new(">>").on_click(|_ctx, data: &mut ApplicationState, _env| {
            data.current_book.go_fast_forward_if_exist();
        });
    let button_prev = Button::new("<").on_click(|_ctx, data: &mut ApplicationState, _env| {
        data.current_book.go_to_prev_page_if_exist();
    });
    let button_fast_back = Button::new("<<").on_click(|_ctx, data: &mut ApplicationState, _env| {
        data.current_book.go_fast_back_if_exist();
    });
    let button_close_book =
        Button::new("close book").on_click(|_ctx, data: &mut ApplicationState, _env| {
            let mut output = OpenOptions::new().append(true).create(true).open("tmp.txt").expect("Unable to open file");
            let mut input=BufReader::new(File::open("file.txt").expect("Cannot open file.txt"));
            for line in input.lines() {
               if !(line.as_ref().unwrap().clone().split_whitespace().next().unwrap().to_string()==data.current_book.get_path())
               {
                   output.write_all((line.unwrap().clone()+"\n").as_bytes()).expect("TODO: panic message");
               }
                else { output.write_all((data.current_book.get_path()+" "+ data.current_book.get_current_chapter_number().to_string().as_str() +" "+data.current_book.get_current_page_number_in_chapter().to_string().as_str()+"\n").as_bytes()); }
            }
            fs::remove_file("file.txt");
            fs::rename("tmp.txt","file.txt");
            data.current_book = Book::empty_book();
        });

    let lbl_num_pag = Label::new(|data: &ApplicationState, _env: &_| {
        format!("{}", data.current_book.get_current_page_number())
    });

    let mut row: Flex<ApplicationState> = Flex::row();
    row.add_child(button_fast_back);
    row.add_child(button_prev);
    row.add_child(button_next);
    row.add_child(button_fast_forward);
    row_due.add_child(lbl_num_pag);

    row.add_child(button_close_book);
    //  col.add_child(row.padding(30.0));

    let page_with_scroll =
        ViewSwitcher::new(
            |data: &ApplicationState, _| data.current_book.current_page.clone(),
            |_, _, _| -> Box<dyn Widget<ApplicationState>> {
                let mut col = Flex::column().cross_axis_alignment(CrossAxisAlignment::Baseline);
                let page =
                    List::new(|| {
                        ViewSwitcher::new(
                            |data: &PageElement, _| data.clone(),
                            |_, data: &PageElement, _| -> Box<dyn Widget<PageElement>> {
                                match data {
                                    PageElement::Text(_) => {
                                        let mut label = RawLabel::new();
                                        label.set_line_break_mode(LineBreaking::WordWrap);
                                        Box::new(label)
                                    }
                                    PageElement::Image(img_buf) => Box::new(Flex::row().with_child(
                                        Image::new(img_buf.clone()).fill_mode(FillStrat::ScaleDown),
                                    )),
                                }
                            },
                        )
                    })
                        .lens(Book::current_page);

                col.add_child(page.padding(30.0).lens(ApplicationState::current_book));
                Box::new(col.scroll().vertical())
            },
        );


    wrapper.add_child(Flex::row().fix_height(8.0));
    wrapper.add_flex_child(row, FlexParams::new(0.07, CrossAxisAlignment::Center));
    wrapper.add_child(Flex::row().fix_height(7.0));
    wrapper.add_flex_child(
        page_with_scroll.fix_width(700.0).fix_height(1000.0),
        FlexParams::new(0.92, CrossAxisAlignment::Baseline),
    );
    wrapper.add_child(Flex::row().fix_height(7.0));
    wrapper.add_flex_child(row_due, FlexParams::new(0.01, CrossAxisAlignment::Center));

    wrapper
}

fn main() {
    const WINDOW_TITLE: LocalizedString<ApplicationState> = LocalizedString::new("ebook reader");
    let mut vet:Vec<String>=Vec::new();//contiene i libri letti in WalkDir
    let mut library:Vec<BookInfo>=Vec::new(); //contiene tutti i libri letti dal file
    let mut i=0;
    let mut name:String=String::new();
    let mut start_chapter:usize=0;
    let mut start_page_in_chapter:usize=0;
    let mut find=0;

    for entry in WalkDir::new("./libri/")
    {
        if ! entry.as_ref().unwrap().path().to_str().unwrap().to_string().eq("./libri/")
        {
            vet.push((*(entry.unwrap().path().to_str().unwrap())).to_string());
        }
    }

    let reader = BufReader::new(File::open("file.txt").expect("Cannot open file.txt"));

    for line in reader.lines() {
        for word in line.unwrap().split_whitespace() {
            if i==0
            {
                name=word.to_string();
                i+=1;
            }
            else if i==1
            {
                start_chapter= usize::from_str_radix(word,10).unwrap();
                i+=1;
            }
            else {
                i=0;
                start_page_in_chapter=usize::from_str_radix(word,10).unwrap();
                library.push(BookInfo{
                    name:name.clone(),
                    start_chapter:start_chapter.clone(),
                    start_page_in_chapter:start_page_in_chapter.clone()
                })
            }
        }
    }

    let mut output = OpenOptions::new().append(true).open("file.txt").expect("Unable to open file");

    for pathElement in vet
    {
        for fileElement in &library
        {
            if fileElement.name.eq(&pathElement.clone())
            {
                find=1;
            }
        }
        if find==0
        { output.write_all((pathElement.clone()+" 0 0\n").as_bytes()).expect("write failed"); }
        else { find=0; }
    }



    // describe the main window
    let main_window = WindowDesc::new(build_widget(library))
        .title(WINDOW_TITLE)
        .window_size((800.0, 1000.0));

    // start the application
    AppLauncher::with_window(main_window)
        .launch(ApplicationState {
            current_book: Book::empty_book(),
        })
        .expect("Failed to launch application");
}
