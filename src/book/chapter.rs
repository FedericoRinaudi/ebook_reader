use crate::book::epub_text::{AttributeCase, EpubText};
use druid::im::HashMap;
use druid::text::{Attribute, RichText};
use druid::{im::Vector, Data, FontStyle, FontWeight, ImageBuf, Lens, FontFamily};
use roxmltree::{Document, Node, ParsingOptions};
use std::path::PathBuf;

use crate::book::page_element::{ContentType, PageElement};
use crate::utilities::get_image_buf;

const MAX_SIZE: f64 = 35.0;

#[derive(Default, Clone, Data, Lens, Debug)]
pub struct Chapter {
    path: String,
    pub xml: String,
    imgs: HashMap<PathBuf, ImageBuf>,
}

impl Chapter {
    pub fn new(path: String, mut xml: String, ebook_path: &str) -> Self {
        xml = xml.replace("&nbsp;", " ");
        let mut imgs = HashMap::new();
        let opt = ParsingOptions { allow_dtd: true };
        let doc = Document::parse_with_options(&xml, opt).unwrap();
        let node = doc.root_element().last_element_child().unwrap();
        Self::fetch_ch_imgs(node, &path, ebook_path, &mut imgs);
        Chapter { path, xml, imgs }
    }

    pub fn format(&self) -> Vector<PageElement> {
        let opt = ParsingOptions { allow_dtd: true };
        let doc = match Document::parse_with_options(&self.xml, opt) {
            Result::Ok(doc) => doc,
            Err(e) => {
                let mut v = Vector::new();
                v.push_back(PageElement::new(ContentType::Error(EpubText::from(e.to_string()))));
                return v;
            }
        };
        let node = doc.root_element().last_element_child().unwrap();
        let mut elements: Vector<PageElement> = Vector::new();
        let mut cur_text = EpubText::new();
        Self::xml_to_elements(node, &mut elements, &mut cur_text, &(*self).imgs);
        elements
    }

    fn fetch_ch_imgs(
        node: Node,
        chapter_path: &str,
        ebook_path: &str,
        imgs: &mut HashMap<PathBuf, ImageBuf>,
    ) {
        if node.tag_name().name() == "img" {
            let ebook_path_buf = PathBuf::from(ebook_path);
            let chapter_path_buf = PathBuf::from(chapter_path);
            let image_path = PathBuf::from(node.attribute("src").unwrap());
            imgs.entry(image_path.clone())
                .or_insert(get_image_buf(&ebook_path_buf, &chapter_path_buf, image_path).unwrap());
        }
        for child in node.children() {
            Self::fetch_ch_imgs(child, chapter_path, ebook_path, imgs);
        }
    }

    fn xml_to_elements(
        node: Node,
        elements: &mut Vector<PageElement>,
        current_text: &mut EpubText,
        images_cache: &HashMap<PathBuf, ImageBuf>,
    ) {
        /* Def Macros */
        macro_rules! recur_on_children {
            () => {
                for child in node.children() {
                    Self::xml_to_elements(child, elements, current_text, images_cache);
                }
            };
        }

        macro_rules! new_line {
            () => {
                elements.push_back(PageElement::new(ContentType::Text(current_text.clone())));
                current_text.reset();
            };
        }

        /*  Actual Transformation */

        if node.is_text() {
            let text = node.text().unwrap();
            let content: Vec<_> = text.split_ascii_whitespace().collect();
            if current_text
                .get_attributes()
                .get(&AttributeCase::FontSize)
                .is_none()
            {
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(16.0)),
                );
            }
            current_text.add_attr(AttributeCase::Style, Attribute::FontFamily(FontFamily::SANS_SERIF));

            if text.starts_with(char::is_whitespace) {
                current_text.push_str(" ");
            }
            current_text.push_str(&content.join(" "));
            if text.ends_with(char::is_whitespace) {
                current_text.push_str(" ");
            }
            current_text.rm_attr(AttributeCase::FontSize);
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
                let image_path = PathBuf::from(node.attribute("src").unwrap());
                elements.push_back(PageElement::new(ContentType::Image(
                    images_cache.get(&image_path).unwrap().clone(),
                )));
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
            "em" | "i" => {
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

    pub fn get_path(&self) -> String {
        (&self).path.clone()
    }
}
