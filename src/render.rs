use roxmltree::{Document, Node, ParsingOptions};
use druid::{im::Vector};
use druid::text::RichText;

fn render_text(n: Node, current_label_text: &mut String, tags: &mut Vector<RichText>) {
    for child in n.children() {
        render(child, current_label_text, tags);
    }
}


fn render(n: Node, current_label_text: &mut String, tags: &mut Vector<RichText>) {
    macro_rules! new_line {
    //TODO: quando  anzi che portarmi dietro una stringa mi porto dietro un rich text propago lo stile alla nuova riga
        () => {
            tags.push_back(RichText::new(current_label_text.as_str().into())); //vado a capo
            current_label_text.replace_range(0.., ""); //resetto la stringa
        };
    }
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
            current_label_text.push_str("****");
            new_line!();
        },
        "img" => {
            new_line!();
            current_label_text.push_str("[IMG]");
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
            render_text(n, current_label_text, tags);
        }
        "em" => {
            //TODO: setto il rich text italic a partire da ora
            render_text(n, current_label_text, tags);
            //TODO: setto il rich text non più italic
        }
        "strong" => {
            //TODO: setto il rich text bold a partire da ora
            render_text(n, current_label_text, tags);
            //TODO: setto il rich text non più bold
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            new_line!();
            //TODO: setto il rich text bold a partire da ora
            render_text(n, current_label_text, tags); //ottengo il text del titolo
            new_line!();
            //TODO: setto il rich text non più bold

        }
        "blockquote" | "div" | "p" | "tr" => {
            // TODO compress newlines
            new_line!();
            render_text(n, current_label_text, tags); //ottengo il text del titolo
            new_line!();
        }
        "li" => {
            new_line!();
            current_label_text.push_str("- ");
            render_text(n, current_label_text, tags); //ottengo il text del titolo
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
        _ => render_text(n, current_label_text, tags),
    }
}

pub fn render_chapter(chapter_str: String) -> Vector<RichText>{
    let opt = ParsingOptions { allow_dtd: true };
    let doc = Document::parse_with_options(&chapter_str, opt).unwrap();
    let body = doc.root_element().last_element_child().unwrap();
    let mut tags :Vector<RichText> = Vector::new();
    let mut str = String::new();
    render(body, &mut str, &mut tags);
    tags
}