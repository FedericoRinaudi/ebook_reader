use crate::book::epub_text::{AttributeCase, EpubText};
use druid::text::{Attribute, RichText};
use druid::{im::Vector, ArcStr, Data, FontStyle, FontWeight, ImageBuf, Lens};
use roxmltree::{Document, Node, ParsingOptions};
use std::io::Read;
use std::path::PathBuf;

use crate::book::page::Page;
use crate::book::page_element::PageElement;

const MAX_SIZE: f64 = 35.0;

#[derive(Default, Clone, Data, Lens, Debug)]
pub struct Chapter {
    path: String,
    xml: String,
}

impl Chapter {
    pub fn new(path: String, mut xml: String) -> Self {
        xml = xml.replace("&nbsp;", " ");
        Chapter { path, xml }
    }

    pub fn format(&self, ebook_path: &str) -> Vector<PageElement> {
        let opt = ParsingOptions { allow_dtd: true };
        let doc = Document::parse_with_options(&chapter_xml, opt).unwrap();
        let node = doc.root_element().last_element_child().unwrap();
        let mut elements: Vector<PageElement> = Vector::new();
        Self::xml_to_elements(node, &self.path, ebook_path, &mut elements, EpubText::new());
        elements
    }

    fn xml_to_elements(
        node: Node,
        chapter_path: &str,
        ebook_path: &str,
        elements: &mut Vector<PageElement>,
        mut current_text: EpubText,
    ) {
        /* Def Macros */
        macro_rules! recur_on_children {
            () => {
                for child in node.children() {
                    Self::xml_to_elements(
                        child,
                        chapter_path,
                        ebook_path,
                        elements,
                        ebook_path,
                        current_text,
                    );
                }
            };
        };

        macro_rules! new_line {
            () => {
                elements.push_back(PageElement::from_text(&current_text));
                current_text.reset();
            };
        }

        /*  Actual Transformation */

        if node.is_text() {
            let text = node.text().unwrap();
            let content: Vec<_> = text.split_ascii_whitespace().collect();
            if text.starts_with(char::is_whitespace) {
                current_text.push_str(" ");
            }
            current_text.push_str(&content.join(" "));
            if text.ends_with(char::is_whitespace) {
                current_text.push_str(" ");
            }
        }

        /* TODO: gestisco gli id
        if let Some(id) = n.attribute("id") {
            c.frag.push((id.to_string(), c.len()));
        }*/
        //TODo: gestione new_line
        match node.tag_name().name() {
            "br" => {
                new_line!();
            }
            "hr" => {
                new_line!();
                current_text.push_str("****");
                new_line!();
            }
            "img" => {
                new_line!();
                //TODO: sposto l'acquisizione dell'immagine in una funzione
                let ebook_path_buf = PathBuf::from(ebook_path);
                let chapter_path_buf = PathBuf::from(chapter_path);
                let image_path = PathBuf::from(n.attribute("src").unwrap());
                let zipfile = std::fs::File::open(ebook_path_buf).unwrap();

                let mut archive = zip::ZipArchive::new(zipfile).unwrap();

                let complete_img_path = unify_paths(chapter_path_buf.clone(), image_path.clone())
                    .into_os_string()
                    .into_string()
                    .unwrap();

                let better_path = convert_path_separators(complete_img_path);

                let mut file = match archive.by_name(&better_path) {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("Error in opening archive at {}", e);
                        return;
                    }
                };

                let mut contents: Vec<u8> = vec![];
                //TODO: match, Err() => Default photo
                file.read_to_end(&mut contents).unwrap();

                elements.push_back(PageElement::from_image(&contents));
                new_line!();
            }
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

            /* TODO: Determinare se sia il caso di gestire diversamente i vari hx */
            "h1" => {
                new_line!();
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!();
            }
            "h2" => {
                new_line!();
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE - 3.00)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!();
            }
            "h3" => {
                new_line!();
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE - 6.00)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!();
            }
            "h4" => {
                new_line!();
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE - 9.00)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!();
            }
            "h5" => {
                new_line!();
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE - 12.00)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!();
            }
            "h6" => {
                new_line!();
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE - 15.00)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!();
            }
            "blockquote" | "div" | "p" | "tr" => {
                // TODO: compress newlines
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

// TODO: sposto in un file utilitiess
fn unify_paths(mut p1: PathBuf, p2: PathBuf) -> PathBuf {
    if !p1.is_dir() {
        p1.pop();
    }
    for el in p2.into_iter() {
        if el == ".." {
            p1.pop();
        } else {
            p1.push(el);
        }
    }
    p1
}

fn convert_path_separators(href: String) -> String {
    let mut path = String::from(href);
    if cfg!(windows) {
        path = path.replace("\\", "/");
        return path;
    }
    path
}
