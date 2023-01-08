use druid::im::{HashMap, Vector};
use druid::text::{Attribute, RichText};
use druid::Data;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Data)]
pub enum AttributeCase {
    FontSize,
    Style,
    Weight,
}

#[derive(Debug, Clone, Data)]
pub struct RangeAttribute {
    #[data(ignore)]
    attribute: Attribute,
    start: usize,
    end: Option<usize>,
}

impl RangeAttribute {
    fn new(attribute: Attribute, start: usize, end: Option<usize>) -> Self {
        Self {
            attribute,
            start,
            end,
        }
    }

    pub(crate) fn get_start(&self) -> &usize {
        &self.start
    }
    pub(crate) fn get_end(&self) -> &Option<usize> {
        &self.end
    }
    pub(crate) fn get_attribute(&self) -> &Attribute {
        &self.attribute
    }
}

#[derive(Debug, Clone, Data)]
pub struct EpubText {
    pub attributes: HashMap<AttributeCase, Vector<RangeAttribute>>,
    pub text: String,
}

impl EpubText {
    pub(crate) fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            text: String::new(),
        }
    }
    pub(crate) fn from(s: String) -> Self {
        Self {
            attributes: HashMap::new(),
            text: s,
        }
    }

    pub(crate) fn get_text(&self) -> &String {
        &self.text
    }

    pub(crate) fn get_attributes(&self) -> &HashMap<AttributeCase, Vector<RangeAttribute>> {
        &self.attributes
    }

    pub(crate) fn add_attr(&mut self, attr_name: AttributeCase, attr: Attribute) {
        self.attributes
            .entry(attr_name)
            .and_modify(|range_attribute| {
                range_attribute.push_back(RangeAttribute::new(
                    attr.clone(),
                    self.text.len(),
                    Option::None,
                ));
            })
            .or_insert(Vector::from(vec![RangeAttribute::new(
                attr,
                self.text.len(),
                Option::None,
            )]));
    }

    pub(crate) fn rm_attr(&mut self, attr_name: AttributeCase) {
        /*
        self.attributes
            .entry(attr_name)
            .and_modify(|range_attribute| match range_attribute.last_mut() { //last mut
                Some(attr) => {
                    (*attr).end = Option::Some(self.text.len());
                }
                None => {}
            });
          */
        self.attributes
            .entry(attr_name)
            .and_modify(|range_attribute| match range_attribute.iter_mut().last() {
                //last mut
                Some(attr) => {
                    (*attr).end = Option::Some(self.text.len());
                }
                None => {}
            });
    }

    pub(crate) fn push_str(&mut self, s: &str) {
        self.text.push_str(s);
    }

    pub(crate) fn reset(&mut self) {
        (*self).text = String::new(); //resetto la stringa
        (*self).attributes = self
            .attributes
            .clone()
            .into_iter()
            .filter(|(_, v)| v.last().unwrap().end.is_none())
            .map(|(key, val)| {
                (
                    key,
                    Vector::from(vec![RangeAttribute::new(
                        (*val.last().unwrap()).attribute.clone(),
                        0 as usize,
                        None,
                    )]),
                )
            })
            .collect();
    }

    /* Crea un PageElement a partire da un EpubText */
    pub fn to_richtext(&self) -> RichText {
        let mut rich_text = RichText::new(self.get_text().as_str().into());
        for range_attributes in self.get_attributes().values() {
            for range_attr in range_attributes {
                match range_attr.get_end() {
                    Some(end) => rich_text.add_attribute(
                        (*range_attr).get_start()..end,
                        range_attr.get_attribute().clone(),
                    ),
                    None => rich_text.add_attribute(
                        (*range_attr).get_start()..,
                        range_attr.get_attribute().clone(),
                    ),
                };
            }
        }
        rich_text
    }
}
