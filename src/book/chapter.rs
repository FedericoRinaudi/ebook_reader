use crate::book::epub_text::{AttributeCase, EpubText};
use crate::book::page_element::ImageState::{Present, Waiting};
use crate::book::page_element::PageElement;
use crate::utilities::{convert_path_separators, get_image_buf, unify_paths};
use druid::im::HashMap;
use druid::text::Attribute;
use druid::{im::Vector, Data, ExtEventSink, FontFamily, FontStyle, FontWeight, ImageBuf, Lens};
use roxmltree::{Document, Node, ParsingOptions};
use std::path::PathBuf;

const MAX_SIZE: f64 = 35.0;

#[derive(Default, Clone, Data, Lens, Debug)]
pub struct Chapter {
    path: String,
    pub xml: String,
    // imgs: HashMap<PathBuf, ImageBuf>,
    pub initial_page: usize,
}

impl Chapter {
    pub fn new(path: String, mut xml: String, initial_page: usize) -> Self {
        xml = xml.replace("&nbsp;", " ");
        xml = xml.replace("&ndash;", "-");
        Chapter {
            path,
            xml,
            initial_page,
        }
    }

    pub fn format(
        &self,
        images_cache: Option<&HashMap<String, ImageBuf>>,
        sink: Option<ExtEventSink>,
        ebook_path: &str,
    ) -> Vector<PageElement> {
        let opt = ParsingOptions { allow_dtd: true };
        let doc = match Document::parse_with_options(&self.xml, opt) {
            Ok(doc) => doc,
            Err(e) => {
                let mut v = Vector::new();
                v.push_back(PageElement::from_error(EpubText::from(e.to_string()), true));
                return v;
            }
        };
        let node = doc.root_element().last_element_child().unwrap();
        let mut elements: Vector<PageElement> = Vector::new();
        let mut cur_text = EpubText::new();
        Self::xml_to_elements(
            node,
            &mut elements,
            &mut cur_text,
            images_cache,
            sink,
            ebook_path,
            &(*self).path,
        );

        elements
    }

    /*fn fetch_ch_imgs(
        node: Node,
        chapter_path: &str,
        ebook_path: &str,
        imgs: &mut HashMap<PathBuf, ImageBuf>,
    ) -> Result<(), ()> {
        if node.tag_name().name() == "img" {
            let ebook_path_buf = PathBuf::from(ebook_path);
            let chapter_path_buf = PathBuf::from(chapter_path);
            let image_path = PathBuf::from( match node.attribute("src") {
                Some(attr) => attr,
                None => return Err(())
            });
            imgs.entry(image_path.clone())
                .or_insert(
                    get_image_buf(ebook_path_buf, &chapter_path_buf, image_path)
                        .unwrap_or(ImageBuf::from_file("./images/default.jpg").unwrap())
                );
        }
        for child in node.children() {
            Self::fetch_ch_imgs(child, chapter_path, ebook_path, imgs);
        }
        Ok(())
    }*/

    fn xml_to_elements(
        node: Node,
        elements: &mut Vector<PageElement>,
        current_text: &mut EpubText,
        images_cache: Option<&HashMap<String, ImageBuf>>,
        sink: Option<ExtEventSink>,
        ebook_path: &str,
        chapter_path: &str,
    ) {
        /* Def Macros */
        macro_rules! recur_on_children {
            () => {
                for child in node.children() {
                    Self::xml_to_elements(
                        child,
                        elements,
                        current_text,
                        images_cache,
                        sink.clone(),
                        ebook_path,
                        chapter_path,
                    );
                }
            };
        }

        macro_rules! new_line {
            ($html: literal) => {
                elements.push_back(PageElement::from_text(
                    current_text.clone(),
                    $html != "HTML",
                ));
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
            current_text.add_attr(
                AttributeCase::Style,
                Attribute::FontFamily(FontFamily::SANS_SERIF),
            );

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
                new_line!("HTML");
            }
            "hr" => {
                //new_line!();
                current_text.push_str("****");
                new_line!("HTML");
            }
            "img" => {
                new_line!("NO_HTML");
                let image_path = String::from(node.attribute("src").unwrap());
                let mut p1 = PathBuf::from(chapter_path);
                p1.pop(); //RIMUOVO IL FILE XML DAL PATH
                let mut complete_img_path =
                    unify_paths(p1, PathBuf::from(&image_path))
                        .into_os_string()
                        .into_string()
                        .unwrap();
                complete_img_path = convert_path_separators(complete_img_path);
                /*LANCIO funzione su altro thread che mi carica il pathbuf*/
                match (images_cache, sink) {
                    (Some(cache), Some(sink)) => elements.push_back(PageElement::from_img_async(
                        match cache.get(&complete_img_path) {
                            Some(refe) => Present(refe.clone()),
                            None => Waiting(complete_img_path.clone()),
                        },
                        false,
                        sink,
                        String::from(ebook_path),
                    )),
                    _ => elements.push_back(PageElement::from_img_sync(
                        Present(
                            match get_image_buf(PathBuf::from(ebook_path), complete_img_path) {
                                None => ImageBuf::from_file("./images/default.jpg").unwrap(),
                                Some(buff) => buff,
                            },
                        ),
                        false,
                    )),
                }
                new_line!("NO_HTML");
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
                new_line!("NO_HTML");
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!("HTML");
            }
            "h2" => {
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                new_line!("NO_HTML");
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE - 3.00)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!("HTML");
            }
            "h3" => {
                new_line!("NO_HTML");
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE - 6.00)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!("HTML");
            }
            "h4" => {
                new_line!("NO_HTML");
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE - 9.00)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!("HTML");
            }
            "h5" => {
                new_line!("NO_HTML");
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE - 12.00)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!("HTML");
            }
            "h6" => {
                new_line!("NO_HTML");
                //TODO: cambio font e fontSize? gestisco il caso in cui il testo fosse già bold?
                current_text.add_attr(AttributeCase::Weight, Attribute::Weight(FontWeight::BOLD));
                current_text.add_attr(
                    AttributeCase::FontSize,
                    Attribute::FontSize(druid::KeyOrValue::Concrete(MAX_SIZE - 15.00)),
                );
                recur_on_children!();
                current_text.rm_attr(AttributeCase::FontSize);
                current_text.rm_attr(AttributeCase::Weight);
                new_line!("HTML");
            }
            "blockquote" | "div" | "p" | "tr" => {
                // TODO: compress newlines
                current_text.push_str("  ");
                recur_on_children!();
                new_line!("HTML");
            }
            "li" => {
                current_text.push_str("  - ");
                recur_on_children!();
                new_line!("HTML");
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
