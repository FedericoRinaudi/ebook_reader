use roxmltree::{Document, Node, ParsingOptions};
use druid::{FontStyle, FontWeight, im::Vector, Data, Lens};
use druid::text::Attribute;
use crate::book::epub_text::{EpubText, AttributeCase};

use crate::book::page::Page;

#[derive(Clone, Data, Lens)]
pub(crate) struct Chapter {
    pages: Vector<Page>,
}

impl Chapter {

    pub(crate) fn new(chapter_xml: String) -> Self {
        let opt = ParsingOptions { allow_dtd: true };
        let doc = Document::parse_with_options(&chapter_xml, opt).unwrap();
        let body = doc.root_element().last_element_child().unwrap();
        Self {
            pages: Self::xml_to_pages(body),
        }
    }

    pub(crate) fn get_page(&self, index: usize) -> Option<Page> {
        self.pages.get(index).map(|page|page.clone())
    }

    pub(crate) fn get_number_of_pages(&self) -> usize {
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
        /*  Def Macros */
        macro_rules! recur_on_children {
            () => {
                for child in n.children() {
                    Self::xml_to_state(child, current_text, pages, current_page);
                }
            }
        }
        macro_rules! new_line {
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

        /*  Actual Transformation */

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