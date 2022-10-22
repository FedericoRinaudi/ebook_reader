use std::{cell::RefCell, sync::Arc};
use std::fs::File;
use std::path::PathBuf;
use druid::ArcStr;
use epub::doc::EpubDoc;
use roxmltree::{Document, Node, ParsingOptions};
use druid::{im::Vector, Widget, LocalizedString, WindowDesc, AppLauncher, Data, Lens, WidgetExt, text::RichText};
use std::rc::Rc;
use druid::widget::{Scroll, Flex, Button, Label, CrossAxisAlignment, List,RawLabel};

#[derive(Clone, Data, Lens)]
struct State {
    chapter: Vector<RichText>,
    epub: Rc<RefCell<EpubDoc<File>>>  //Da spostare (forse) in env
}
/*
#[derive(Clone, Data, Lens)]
pub struct RichText {
    content: String
}


impl RichText {
    pub fn new(text: &str) -> Self {
        Self {
            content: text.to_string()
        }
    }
}
*/

fn render_RichText(n: Node, current_label_text: &mut String, RichTexts: &mut Vector<RichText>) {
    if n.is_text() {
        let text = n.text().unwrap();
        let content: Vec<_> = text.split_ascii_whitespace().collect();

        if text.starts_with(char::is_whitespace) {
            current_label_text.push(' ');
        }
        current_label_text.push_str(&content.join(" "));
        if text.ends_with(char::is_whitespace) {
            current_label_text.push(' ');
        }
        RichTexts.push_back(RichText::new(arcstr::literal!(&current_label_text)));
        current_label_text.replace_range(0.., "");
    }
    /*
    if let Some(id) = n.attribute("id") {
        c.frag.push((id.to_string(), c.len()));
    }*/

    match n.RichText_name().name() {
        "br" => RichTexts.push_back(RichText::new(arcstr::literal!("****"))),
        "hr" => RichTexts.push_back(RichText::new(arcstr::literal!("****"))),
        "img" =>  {},
        "a" => {
            /*
            match n.attribute("href") {
                // TODO open external urls in browser
                Some(url) if !url.starts_with("http") => {
                    let start = c.len();
                    c.render(n, Attribute::Underlined, Attribute::NoUnderline);
                    c.links.push((start, c.len(), url.to_string()));
                }
                _ => c.render_text(n),
            }*/
        }
        "em" => {},// c.render(n, Attribute::Italic, Attribute::NoItalic),
        "strong" => {},// c.render(n, Attribute::Bold, Attribute::NormalIntensity),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            //c.push('\n');
            //c.render(n, Attribute::Bold, Attribute::NormalIntensity);
            //c.push('\n');
        }
        "li" => {
            current_label_text.push_str("- ");
        }
        "pre" => {
            /*c.push_str("\n  ");
            n
                .descendants()
                .filter(Node::is_text)
                .map(|n| n.text().unwrap().replace('\n', "\n  "))
                .for_each(|s| c.push_str(&s));
            c.push('\n');*/
        }
        _ => {},
    };
}

fn render_all_child_RichTexts(root: Node, text: &mut String, RichTexts: &mut Vector<RichText>){
    render_RichText(root, text, RichTexts);
    for child in root.children(){
        render_all_child_RichTexts(child, text, RichTexts)
    }
}

fn render_chapter(chapter_str: String) -> Vector<RichText>{
    let opt = ParsingOptions { allow_dtd: true };
    let doc = Document::parse_with_options(&chapter_str, opt).unwrap();
    let body = doc.root_element().last_element_child().unwrap();
    let mut RichTexts :Vector<RichText> = Vector::new();
    let mut str = String::new();
    render_all_child_RichTexts(body, &mut str, &mut RichTexts);
    RichTexts
}



fn build_widget() -> impl Widget<State> {
    let mut col = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
    let buttonNext = Button::new("next page").on_click(|_ctx, data: &mut State, _env| {
        if data.epub.borrow_mut().go_next().is_ok(){
            data.chapter = render_chapter(data.epub.borrow_mut().get_current_str().unwrap());
        }
    });
    let buttonPrev = Button::new("prev page").on_click(|_ctx, data: &mut State, _env| {
        if data.epub.borrow_mut().go_prev().is_ok(){
            data.chapter = render_chapter(data.epub.borrow_mut().get_current_str().unwrap());
        }
    });
    let mut row:Flex<State>=Flex::row();
    row.add_child(buttonPrev);
    row.add_child(buttonNext);
   
    col.add_child(row);
    let page = List::new(||{
          //     let x:RawLabel<RichText>= RawLabel::new();
            Label::new(|item: &RichText, _env: &_| item.content.clone())
        }
    ).lens(State::chapter);
    col.add_child(page);
    Scroll::new(col)
}


fn main() {


    //let mut epub = Arc::new(Mutex::new(EpubDoc::new(PathBuf::from("./sample.epub")).unwrap()));
    let epub = Rc::new(RefCell::new(EpubDoc::new(PathBuf::from("./libro.epub")).unwrap()));
    //const VERTICAL_WIDGET_SPACING: f64 = 20.0;
    //const TEXT_BOX_WIDTH: f64 = 200.0;
    const WINDOW_TITLE :LocalizedString<State> = LocalizedString::new("Hello World!");
    // describe the main window
    let main_window = WindowDesc::new(build_widget)
        .title(WINDOW_TITLE)
        .window_size((800.0, 800.0));


    // create the initial app state
    let initial_state = State {
        chapter: render_chapter(epub.borrow_mut().get_current_str().unwrap()),
        epub: epub.clone(),
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");

}