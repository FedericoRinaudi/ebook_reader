mod book;

use std::fs;
use std::fs::{ File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use druid::widget::{Button, CrossAxisAlignment, FillStrat, Flex, FlexParams, Image, Label, LineBreaking, List, RawLabel, ViewSwitcher, Controller, ControllerHost, Click, TextBox};
use druid::{AppLauncher, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc, EventCtx, Event, Env, ImageBuf, lens, LensExt};
use std::path::PathBuf;
use druid::im::Vector;
use epub::doc::EpubDoc;
use walkdir::WalkDir;


use crate::book::page_element::PageElement;
use crate::book::Book;



#[derive(Default, Clone, Data, Lens)]
pub struct ApplicationState {
    pub current_book: Book,
    library: Vector<BookInfo>
}
#[derive(Default, Clone, Data, Lens, Debug)]
pub struct BookInfo {
    name:String,
    start_chapter:usize,
    start_page_in_chapter:usize,
    tot_pages:usize,
    image:String
}

struct TakeFocus;

impl<T, W: Widget<T>> Controller<T, W> for TakeFocus {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::WindowConnected = event {
            ctx.request_focus();
        }
        child.event(ctx, event, data, env)
    }
}


//SWITCH TRA VISUALIZZATORE ELENCO EBOOK E VISUALIZZATORE EBOOK
fn build_widget<'a>() -> impl Widget<ApplicationState> {
    let a: ViewSwitcher<ApplicationState, bool> = ViewSwitcher::new(
        |data: &ApplicationState, _| data.current_book.is_empty(),
        |_ctx, data: &ApplicationState, _env| -> Box<dyn Widget<ApplicationState>>{
            if data.current_book.is_empty() {
                let mut col = Flex::column();
                let mut row = Flex::row();
                let mut lib = data.library.clone();
                let row_flex = 1.0/((lib.len() as f64 /3.0) +1.0);
                for (i, e) in lib.into_iter().enumerate() {

                    //println!("{:?}", e);

                /*    let b = Button::new(e.name.clone()).on_click(move |_ctx, button_data: &mut ApplicationState, _env| {
                        button_data.current_book = Book::new(PathBuf::from(e.name.clone()), e.start_chapter, e.start_page_in_chapter, e.tot_pages).unwrap();
                    });
                    */
                    let b=ImageBuf::from_file(e.image.clone()).unwrap();
                    let c=ControllerHost::new(Image::new(b).fix_width(300.0).fix_height(200.0),Click::new(move |_ctx, data: &mut ApplicationState, _env| {
                        data.current_book = Book::new(PathBuf::from(e.name.clone()), e.start_chapter, e.start_page_in_chapter, e.tot_pages).unwrap();
                    }));
                    row.add_flex_child(c, FlexParams::new(row_flex, CrossAxisAlignment::Start));
                    if i != 0 && (i+1)%3 == 0{
                        col.add_flex_child(row, FlexParams::new(0.3, CrossAxisAlignment::Center));
                        row = Flex::row();
                    }
                }
                col.add_child(row);
                Box::new(col.scroll().vertical())
            } else {
                  return Box::new(render_book())
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
            let mut output = OpenOptions::new().append(true).create(true).open("./tmp.txt").expect("Unable to open file");
            let  input=BufReader::new(File::open("file.txt").expect("Cannot open file.txt"));
            for line in input.lines() {
               if !(line.as_ref().unwrap().clone().split_whitespace().next().unwrap().to_string()==data.current_book.get_path())
               {
                   output.write_all((line.unwrap().clone()+"\n").as_bytes()).expect("TODO: panic message");
               }
                else {
                    let _ = output.write_all((
                        data.current_book.get_path()+" "+
                        data.current_book.get_current_chapter_number().to_string().as_str() +" "+
                        data.current_book.get_current_page_number_in_chapter().to_string().as_str()+" "+
                        data.current_book.get_current_page_number().to_string().as_str()+" "+
                        data.current_book.get_image(data.current_book.get_path()).to_string().as_str()+"\n").as_bytes());}

            }
            let _ = fs::remove_file("file.txt");
            let _ = fs::rename("tmp.txt","file.txt");

            data.library= read_from_file();
            data.current_book = Book::empty_book();

        });

    let switch_mode = ViewSwitcher::new(
        |data: &ApplicationState, _| data.current_book.clone(),
        |_, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
            let tag:&str;
            if data.current_book.edit {tag = "Read"} else {tag = "Edit"};
            let switch = Button::new(tag).on_click(|_ctx, data: &mut ApplicationState, _env| {
                if data.current_book.edit {data.current_book.save_n_update()};
                data.current_book.edit = !data.current_book.edit;
            });
            Box::new(switch)
        });

    let lbl_num_pag = Label::new(|data: &ApplicationState, _env: &_| {
        format!("{}", data.current_book.get_current_page_number())
    });

    let mut row: Flex<ApplicationState> = Flex::row();
    row.add_child(button_fast_back);
    row.add_child(button_prev);
    row.add_child(button_next);
    row.add_child(button_fast_forward);
    row.add_child(switch_mode);
    row_due.add_child(lbl_num_pag);

    row.add_child(button_close_book);
    //  col.add_child(row.padding(30.0));

    let page_with_scroll =
        ViewSwitcher::new(
            |data: &ApplicationState, _| data.current_book.edit,
            |_, data: &ApplicationState, _| -> Box<dyn Widget<ApplicationState>> {
                let mut col = Flex::column().cross_axis_alignment(CrossAxisAlignment::Baseline);
                if !data.current_book.edit {
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
                        }).lens(Book::current_page);
                    col.add_child(page.padding(30.0).lens(ApplicationState::current_book));
                }else{

                    let xml_lens  = lens!(ApplicationState, current_book)
                        .then(lens!(Book, chapters_xml_and_path))
                        .index(data.current_book.current_chapter_number)
                        .then(lens!((String, String), 0));

                    let text = TextBox::new()
                        .with_line_wrapping(true)
                        .lens(xml_lens);

                    col.add_child(text);
                }
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
    let mut library:Vector<BookInfo>=Vector::new();//contiene tutti i libri letti dal file

    let mut find=0;

    for entry in WalkDir::new("./libri/").into_iter().skip(1)
    {
        vet.push((*(entry.unwrap().path().to_str().unwrap())).to_string());

    }

    library= read_from_file();

   

    let mut output = OpenOptions::new().append(true).open("file.txt").expect("Unable to open file");

    for path_element in vet
    {
        for file_element in &library
        {
            if file_element.name.eq(&path_element.clone())
            {
                find=1;
            }
        }
        if find==0
        {
            let image=get_image(path_element.clone());
            output.write_all((path_element.clone()+" 0 0 0 "+image.as_str()+ "\n").as_bytes()).expect("write failed");
        }
        else { find=0; }
    }
    library= read_from_file();



    // describe the main window
    let main_window = WindowDesc::new(build_widget())
        .title(WINDOW_TITLE)
        .window_size((800.0, 1000.0));

    // start the application
    AppLauncher::with_window(main_window)
        .launch(ApplicationState {
            current_book: Book::empty_book(),
            library:library
        })
        .expect("Failed to launch application");




}

fn read_from_file() ->Vector<BookInfo>
{
    let mut library:Vector<BookInfo>=Vector::new();//contiene tutti i libri letti dal file
    let reader = BufReader::new(File::open("file.txt").expect("Cannot open file.txt"));
    let mut name:String=String::new();
    let mut start_chapter:usize=0;
    let mut start_page_in_chapter:usize=0;
    let mut tot_pages:usize=0;
    let mut i=0;

   
    for line in reader.lines()
    {
        let mut word=line.as_ref().unwrap().split_whitespace().into_iter();
            library.push_back(BookInfo{
                //name:word.next().unwrap().to_string().clone(),
                name:word.next().unwrap().to_string().clone(),
                start_chapter:usize::from_str_radix(word.next().unwrap(),10).unwrap(),
                start_page_in_chapter:usize::from_str_radix(word.next().unwrap(),10).unwrap(),
                tot_pages:usize::from_str_radix(word.next().unwrap(),10).unwrap(),
                image:word.next().unwrap().to_string().clone()

            })
        
           
    }
    return library;
}


fn get_image(bookPath:String)->String
{
    let doc = EpubDoc::new(bookPath);
    assert!(doc.is_ok());
    let mut doc = doc.unwrap();
    let name=doc.mdata("cover").unwrap();
    let title=doc.mdata("title").unwrap().replace(" "  ,"_") .split('/').into_iter().next().unwrap().to_string();

    let cover_data = doc.get_cover().unwrap();

    let mut path=String::from("./images/");
    path.push_str(title.as_str());
    path.push_str(".jpeg");

    let f = fs::File::create(path.clone());
    assert!(f.is_ok());
    let mut f = f.unwrap();
    let resp = f.write_all(&cover_data);

    return path;

}