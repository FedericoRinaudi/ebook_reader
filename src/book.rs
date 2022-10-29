use roxmltree::{Document, Node, ParsingOptions};
use druid::{FontStyle, FontWeight, im::Vector, Data, Lens};
use druid::text::{RichText, Attribute};
use std::collections::HashMap;
use std::option::Option::{Some, None};
use std::path::Path;
use druid::widget::ListIter;
use unicode_segmentation::UnicodeSegmentation;
use epub::doc::EpubDoc;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AttributeCase {
    Style,
    Weight
    //TODO: aggiungo e aggiorno i casi man mano che mi servono
}


#[derive(Debug, Clone)]
struct RangeAttribute {
    attribute: Attribute,
    start: usize,
    end: Option<usize>
}

impl RangeAttribute {

    fn new(attribute: Attribute, start: usize, end: Option<usize>) -> Self {
        Self{
            attribute,
            start,
            end
        }
    }

}


#[derive(Debug, Clone)]
struct EpubText {
    attributes: HashMap<AttributeCase, Vec<RangeAttribute>>,
    text: String
}

impl EpubText {

    fn new() -> Self {
        Self{
            attributes: HashMap::new(),
            text: String::new()
        }
    }

    fn add_attr(&mut self, attr_name: AttributeCase, attr: Attribute){
        self.attributes.entry(attr_name)
            .and_modify(|range_attribute|{
                range_attribute.push(RangeAttribute::new(attr.clone(), self.text.len(), Option::None));
            })
            .or_insert(vec![RangeAttribute::new(attr, self.text.len(), Option::None)]);
    }

    fn rm_attr(&mut self, attr_name: AttributeCase){
        self.attributes.entry(attr_name)
            .and_modify(|range_attribute|{
                match range_attribute.last_mut() {
                    Some(attr) => {
                        (*attr).end = Option::Some(self.text.len());
                    },
                    None => {}
                }
            });
    }

    fn push_str(&mut self, s: &str){
        self.text.push_str(s);
    }

    fn reset(&mut self){
        (*self).text = String::new(); //resetto la stringa
        (*self).attributes = self.attributes.clone().into_iter()
            .filter(|(_, v)| v.last().unwrap().end.is_none())
            .map(|(key, val)|(key, vec![RangeAttribute::new((*val.last().unwrap()).attribute.clone(), 0 as usize, None)]))
            .collect();
    }

}


const MAX_PAGE_LINES: usize = 42;


#[derive(Clone, Data, Lens)]
pub struct Page {
    page: Vector<RichText>,
    num_lines: usize
}

impl Page {
    fn new() -> Self {
        Self {
            page: Vector::new(),
            num_lines: 0
        }
    }
    fn add_lines(&mut self, text: & EpubText) -> Result<(), ()> {
        let text_estimated_lines = (text.text.graphemes(true).count() / 100) + 1 ;
        if (*self).num_lines != 0 && ((text_estimated_lines + (*self).num_lines) > MAX_PAGE_LINES) {
            return Err(());
        };
        let mut rich_text = RichText::new(text.text.as_str().into());
        for range_attributes in text.attributes.values(){
            for range_attr in range_attributes{
                match range_attr.end {
                    Some(end) => rich_text.add_attribute((*range_attr).start..end, range_attr.attribute.clone()),
                    None => rich_text.add_attribute((*range_attr).start.., range_attr.attribute.clone()),
                };
            }
        }
        self.page.push_back(rich_text);
        (*self).num_lines += text_estimated_lines;
        Ok(())

    }
}

impl ListIter<RichText> for Page {
    fn for_each(&self, cb: impl FnMut(&RichText, usize)) {
        self.page.for_each(cb);
    }

    fn for_each_mut(&mut self, cb: impl FnMut(&mut RichText, usize)) {
        self.page.for_each_mut(cb);
    }

    fn data_len(&self) -> usize {
        self.page.data_len()
    }
}

#[derive(Clone, Data, Lens)]
struct Chapter {
    pages: Vector<Page>,
}

impl Chapter {

    fn new(chapter_xml: String) -> Self {
        let opt = ParsingOptions { allow_dtd: true };
        let doc = Document::parse_with_options(&chapter_xml, opt).unwrap();
        let body = doc.root_element().last_element_child().unwrap();
        Self {
            pages: Self::xml_to_pages(body),
        }
    }

    fn get_page(&self, index: usize) -> Option<Page> {
        self.pages.get(index).map(|page|page.clone())
    }

    fn get_number_of_pages(&self) -> usize {
        self.pages.len()
    }

    fn xml_to_pages(body: Node) -> Vector<Page> {
        let mut pages :Vector<Page> = Vector::new();
        let mut current_text = EpubText::new();
        let mut current_page = Page::new();
        Self::xml_to_state(body, &mut current_text, &mut pages, &mut current_page);
        pages.push_back(current_page);
        pages
    }

    fn xml_to_state(n: Node, current_text: &mut EpubText, pages: &mut Vector<Page>, current_page: &mut Page) {
        macro_rules! recur_on_children {
            () => {
                for child in n.children() {
                    Self::xml_to_state(child, current_text, pages, current_page);
                }
            }
        }
        macro_rules! new_line {
    //TODO: quando  anzi che portarmi dietro una stringa mi porto dietro un rich text propago lo stile alla nuova riga
            () => {
                match current_page.add_lines(current_text){
                    Ok(_) => {}
                    Err(_) => {
                        pages.push_back(current_page.clone());
                        *current_page = Page::new();
                        current_page.add_lines(current_text).unwrap();
                    }
                }
                current_text.reset();
            };
        }

        if n.is_text() {
            let text = n.text().unwrap();
            let content: Vec<_> = text.split_ascii_whitespace().collect();
            if text.starts_with(char::is_whitespace) {
                current_text.push_str(" ");
            }
            current_text.push_str(&content.join(" "));
            if text.ends_with(char::is_whitespace) {
                current_text.push_str(" ");
            }
        }
        //TODO: gestisco gli id
        /*
        if let Some(id) = n.attribute("id") {
            c.frag.push((id.to_string(), c.len()));
        }*/
        match n.tag_name().name() {
            "br" => {
                new_line!();
            },
            "hr" => {
                new_line!();
                current_text.push_str("****");
                new_line!();
            },
            "img" => {
                new_line!();
                current_text.push_str("[IMG]");
                new_line!();
            } ,
            "a" => {
                /*match n.attribute("href") {
                    // TODO open external urls in browser
                    Some(url) if !url.starts_with("http") => {
                        let start = c.text.len();
                        c.render(n, Attribute::Underlined, Attribute::NoUnderline);
                        c.links.push((start, c.text.len(), url.to_string()));
                    }
                    _ => c.render_text(n),
                }*/
                //TODO: gestisco il tag prima di ricorrere
                recur_on_children!();
            }
            "em" => {
                //TODO: aggiungo le righe commentate se penso sia il caso di gestire il caso in cui sia presente il tag 'em' nonstante il font fosse già italic
                //let prev_style = current_rich_text.attributes.get(AttributeCase::Style).map(|el|{(*el).clone()});
                current_text.add_attr(AttributeCase::Style, Attribute::Style(FontStyle::Italic));
                recur_on_children!();
                current_text.rm_attr(AttributeCase::Style);
                /*match prev_style {
                    Some(p_s) => current_rich_text.add_attr("Style".to_string(), p_s.attribute),
                    None => {}
                }*/
            }
            "strong" => {
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                recur_on_children!();
                current_text.rm_attr(AttributeCase::Weight);
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                new_line!();
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                recur_on_children!();
                current_text.rm_attr(AttributeCase::Weight);
                new_line!();

            }
            "blockquote" | "div" | "p" | "tr" => {
                // TODO compress newlines
                new_line!();
                recur_on_children!();
                new_line!();
            }
            "li" => {
                new_line!();
                current_text.push_str("- ");
                recur_on_children!();
                new_line!();
            }
            //TODO: implementare tag pre
            /*"pre" => {
                c.text.push_str("\n  ");
                n
                    .descendants()
                    .filter(Node::is_text)
                    .map(|n| n.text().unwrap().replace('\n', "\n  "))
                    .for_each(|s| c.text.push_str(&s));
                c.text.push('\n');
            }*/
            _ => recur_on_children!(),
        }
    }

}

#[derive(Clone, Data, Lens)]
pub struct Book {
    chapters_xml: Vector<String>,
    current_chapter_number: usize,
    current_page_number_in_chapter: usize,
    current_chapter: Chapter,
    pub current_page: Page
}


impl Book {

    pub fn new<P: AsRef<Path>>(path: P, initial_chapter_number: usize, initial_page_number_in_chapter: usize) -> Result<Self, ()> {
        let mut epub_doc = match EpubDoc::new(path){
            Ok(epub) => epub,
            Err(_) => return Result::Err(())
        };

        let mut chapters_xml = Vector::new();

        while {
            //TODO: gestisco diversamente l'unwrap... qua in effetti spesso va in errore
            let chapter_xml = epub_doc.get_current_str().unwrap();
            chapters_xml.push_back(chapter_xml);
            epub_doc.go_next().is_ok()
        } {}

        let initial_chapter = Chapter::new(match chapters_xml.get(initial_chapter_number){
            Some(chapter_xml) => chapter_xml.clone(),
            None => return Err(())
        });

        let initial_page = match initial_chapter.get_page(initial_page_number_in_chapter){
            Some(page) => page,
            None => return Err(())
        };

        Result::Ok(
            //TODO: gestisco diversamente gli unwrap (se per esempio avessi il primo capitolo vuoto si spaccherebbe tutto, è corretto?)
            Self {
                chapters_xml,
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
            if (*self).chapters_xml.get((*self).current_chapter_number + 1).is_some() { //NON SONO ALL'ULTIMO CAPITOLO?
                (*self).current_chapter_number += 1;
                (*self).current_page_number_in_chapter = 0;
                (*self).current_chapter = Chapter::new(self.chapters_xml.get((*self).current_chapter_number).unwrap().clone());
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
                (*self).current_chapter = Chapter::new(self.chapters_xml.get((*self).current_chapter_number).unwrap().clone());
                (*self).current_page_number_in_chapter = self.current_chapter.get_number_of_pages();
            } else {
                return;
            }
        }
        (*self).current_page_number_in_chapter = (*self).current_page_number_in_chapter - 1;
        (*self).current_page = self.current_chapter.get_page((*self).current_page_number_in_chapter).unwrap();
    }

}
