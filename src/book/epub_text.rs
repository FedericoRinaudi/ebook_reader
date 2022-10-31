use druid::im::HashMap;
use druid::text::Attribute;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum AttributeCase {
    Style,
    Weight, //TODO: aggiungo e aggiorno i casi man mano che mi servono
}

#[derive(Debug, Clone)]
pub(crate) struct RangeAttribute {
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

#[derive(Debug, Clone)]
pub struct EpubText {
    attributes: HashMap<AttributeCase, Vec<RangeAttribute>>,
    text: String,
}

impl EpubText {
    pub(crate) fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            text: String::new(),
        }
    }

    pub(crate) fn get_text(&self) -> &String {
        &self.text
    }

    pub(crate) fn get_attributes(&self) -> &HashMap<AttributeCase, Vec<RangeAttribute>> {
        &self.attributes
    }

    pub(crate) fn add_attr(&mut self, attr_name: AttributeCase, attr: Attribute) {
        self.attributes
            .entry(attr_name)
            .and_modify(|range_attribute| {
                range_attribute.push(RangeAttribute::new(
                    attr.clone(),
                    self.text.len(),
                    Option::None,
                ));
            })
            .or_insert(vec![RangeAttribute::new(
                attr,
                self.text.len(),
                Option::None,
            )]);
    }

    pub(crate) fn rm_attr(&mut self, attr_name: AttributeCase) {
        self.attributes
            .entry(attr_name)
            .and_modify(|range_attribute| match range_attribute.last_mut() {
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
                    vec![RangeAttribute::new(
                        (*val.last().unwrap()).attribute.clone(),
                        0 as usize,
                        None,
                    )],
                )
            })
            .collect();
    }
}
