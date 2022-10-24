use roxmltree::{Document, Node, ParsingOptions};
use druid::{FontStyle, FontWeight, im::Vector};
use druid::text::{RichText, Attribute};
use std::collections::HashMap;
use std::option::Option::{Some, None};

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

struct CurrentRichText {
    attributes: HashMap<String, Vec<RangeAttribute>>,
    text: String
}

impl CurrentRichText {

    fn new() -> Self {
        Self{
            attributes: HashMap::new(),
            text: String::new()
        }
    }

    fn add_attr(&mut self, attr_name: String, attr: Attribute){
        self.attributes.entry(attr_name)
            .and_modify(|range_attribute|{
                    range_attribute.push(RangeAttribute::new(attr.clone(), self.text.len(), Option::None));
                })
            .or_insert(vec![RangeAttribute::new(attr, self.text.len(), Option::None)]);
    }

    fn rm_attr(&mut self, attr_name: String){
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

}

fn render_text(n: Node, current_rich_text: &mut CurrentRichText, rich_texts: &mut Vector<RichText>) {
    for child in n.children() {
        render(child, current_rich_text, rich_texts);
    }
}


fn render(n: Node, current_rich_text: &mut CurrentRichText, rich_texts: &mut Vector<RichText>) {
    macro_rules! new_line {
    //TODO: quando  anzi che portarmi dietro una stringa mi porto dietro un rich text propago lo stile alla nuova riga
        () => {
            let mut rich_text = RichText::new(current_rich_text.text.as_str().into());
            for range_attributes in current_rich_text.attributes.values(){
                for range_attr in range_attributes{
                    match range_attr.end {
                        Some(end) => rich_text.add_attribute((*range_attr).start..end, range_attr.attribute.clone()),
                        None => rich_text.add_attribute((*range_attr).start.., range_attr.attribute.clone()),
                    };
                }
            }
            //TODO: rimuovo gli unwrap e gestisco il caso di errore
            rich_texts.push_back(rich_text); //vado a capo
            current_rich_text.text.replace_range(0.., ""); //resetto la stringa
            (*current_rich_text).attributes = current_rich_text.attributes.clone().into_iter()
                .filter(|(_, v)| v.last().unwrap().end.is_none())
                .map(|(key, val)|(key, vec![RangeAttribute::new((*val.last().unwrap()).attribute.clone(), 0 as usize, None)]))
                .collect();
        };
    }


    if n.is_text() {
        let text = n.text().unwrap();
        let content: Vec<_> = text.split_ascii_whitespace().collect();
        if text.starts_with(char::is_whitespace) {
            current_rich_text.text.push(' ');
        }
        current_rich_text.text.push_str(&content.join(" "));
        if text.ends_with(char::is_whitespace) {
            current_rich_text.text.push(' ');
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
            current_rich_text.text.push_str("****");
            new_line!();
        },
        "img" => {
            new_line!();
            current_rich_text.text.push_str("[IMG]");
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
            render_text(n, current_rich_text, rich_texts);
        }
        "em" => {
            //TODO: aggiungo le righe commentate se penso sia il caso di gestire il caso in cui sia presente il tag 'em' nonstante il font fosse già italic
            //let prev_style = current_rich_text.attributes.get("Style").map(|el|{(*el).clone()});
            current_rich_text.add_attr("Style".to_string(), Attribute::Style(FontStyle::Italic));
            render_text(n, current_rich_text, rich_texts);
            current_rich_text.rm_attr("Style".to_string());
            /*match prev_style {
                Some(p_s) => current_rich_text.add_attr("Style".to_string(), p_s.attribute),
                None => {}
            }*/
        }
        "strong" => {
            current_rich_text.add_attr("Weight".to_string(), Attribute::Weight(FontWeight::BOLD));
            render_text(n, current_rich_text, rich_texts);
            current_rich_text.rm_attr("Weight".to_string());
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            new_line!();
            //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
            current_rich_text.add_attr("Weight".to_string(), Attribute::Weight(FontWeight::BOLD));
            render_text(n, current_rich_text, rich_texts);
            current_rich_text.rm_attr("Weight".to_string());
            new_line!();

        }
        "blockquote" | "div" | "p" | "tr" => {
            // TODO compress newlines
            new_line!();
            render_text(n, current_rich_text, rich_texts);
            new_line!();
        }
        "li" => {
            new_line!();
            current_rich_text.text.push_str("- ");
            render_text(n, current_rich_text, rich_texts);
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
        _ => render_text(n, current_rich_text, rich_texts),
    }
}

pub fn render_chapter(chapter_str: String) -> Vector<RichText>{
    let opt = ParsingOptions { allow_dtd: true };
    let doc = Document::parse_with_options(&chapter_str, opt).unwrap();
    let body = doc.root_element().last_element_child().unwrap();
    let mut rich_texts :Vector<RichText> = Vector::new();
    let mut rich_text = CurrentRichText::new();
    render(body, &mut rich_text, &mut rich_texts);
    rich_texts
}