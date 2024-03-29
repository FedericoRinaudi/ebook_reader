use crate::algorithms::OcrAlgorithms;
use crate::book::chapter::Chapter;
use crate::utilities::xml_to_text;
use druid::{im::Vector, Data, Lens};
use regex;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Copy, Clone, Default, Data, Lens, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mapping {
    pub page: usize,              //Page number
    pub(crate) is_first: bool,    //E' la prima pagina di un capitolo?
    pub(crate) tot_chars: usize,  //Total sum of characters to get the average
    pub(crate) full_lines: usize, //Number of lines considered to get average characters
    pub page_lines: usize,
}

impl Mapping {
    pub fn new(str: String) -> Result<Self, Box<dyn Error>> {
        let mut init = Mapping {
            page: 0,
            is_first: false,
            tot_chars: 0,
            full_lines: 0,
            page_lines: 0,
        };
        init.page_stats(str)?;
        Ok(init)
    }

    fn page_stats(&mut self, str: String) -> Result<(), Box<dyn Error>> {
        //ARRAY CON UNA LINEA PER OGNI RIGA
        let mut lines = str
            .split("\n")
            .filter(|s| {
                if s.graphemes(true).count() < 4 {
                    if let Some(pg_num) = Regex::new(r"\d+").unwrap().captures(s) {
                        self.page = pg_num[0].parse::<usize>().unwrap();
                    }
                    return false;
                } else {
                    return true;
                }
            })
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        self.page_lines = lines.len();

        lines = lines
            .iter()
            .filter(|s| {
                let last = s.chars().last().unwrap();
                last.is_alphabetic() || last == '-'
            })
            .map(|s| s.clone())
            .collect::<Vec<String>>();

        let (mut sum, mut count) = lines
            .iter()
            .map(|s| s.graphemes(true).count())
            .fold((0, 0), |(sum, count), value| (sum + value, count + 1));

        let first_avg = sum as f64 / count as f64;

        (sum, count) = lines
            .iter()
            .filter(|s| {
                s.trim().graphemes(true).count() as f64 > first_avg - 5.
            })
            .map(|s| s.graphemes(true).count())
            .fold((0, 0), |(sum, count), value| (sum + value, count + 1));

        self.full_lines = count;
        self.tot_chars = sum;
        Ok(())
    }
}

#[derive(Default, Clone, Data, Lens, PartialEq, Debug)]
pub struct OcrData {
    pub mappings: Vector<Mapping>,
    pub first_chap: Option<usize>,
    pub first: Option<usize>,
    pub other: Option<usize>,
}

#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SerializableOcrData {
    pub mappings: Vec<Mapping>,
    pub first_chap: Option<usize>,
    pub first: Option<usize>,
    pub other: Option<usize>,
}

impl From<OcrData> for SerializableOcrData {
    fn from(ocr_data: OcrData) -> Self {
        SerializableOcrData {
            mappings: ocr_data.mappings.iter().map(|m| *m).collect(),
            first_chap: ocr_data.first_chap,
            first: ocr_data.first,
            other: ocr_data.other,
        }
    }
}

impl From<SerializableOcrData> for OcrData {
    fn from(s_ocr_data: SerializableOcrData) -> Self {
        OcrData {
            mappings: s_ocr_data.mappings.iter().map(|m| *m).collect(),
            first_chap: s_ocr_data.first_chap,
            first: s_ocr_data.first,
            other: s_ocr_data.other,
        }
    }
}

impl OcrData {
    pub fn new() -> Self {
        OcrData {
            mappings: Vector::new(),
            first_chap: None,
            first: None,
            other: None,
        }
    }

    pub fn ocr_log(&mut self, str: String) -> Result<usize, ()> {
        if self.first.is_none() || self.other.is_none() {
            return Err(());
        }
        match Mapping::new(str) {
            Ok(mut mapping) => {
                let first_lines = self.mappings[self.first.unwrap()].page_lines;
                let other_lines = self.mappings[self.other.unwrap()].page_lines;
                let range = first_lines - 2..first_lines + 3;
                let other_range = other_lines - 2..other_lines + 3;
                if range.contains(&mapping.page_lines) {
                    mapping.is_first = true
                } else if !other_range.contains(&mapping.page_lines) {
                    return Err(());
                }
                self.mappings.push_back(mapping);

                return Ok(&self.mappings.len() - 1);
            }
            Err(e) => {
                eprintln!("{}", e);
                Err(())
            }
        }
    }

    pub fn ocr_log_first(&mut self, str: String, ch: usize) -> Result<(), Box<dyn Error>> {
        match Mapping::new(str) {
            Ok(mut mapping) => {
                mapping.is_first = true;
                self.mappings.push_back(mapping);
                self.first = Some(&self.mappings.len() - 1);
                self.first_chap = Some(ch);
                return Ok(());
            }
            Err(e) => {
                eprintln!("{:?}", e);
                Err(e)
            }
        }
    }

    pub fn ocr_log_other(&mut self, str: String) -> Result<(), Box<dyn Error>> {
        match Mapping::new(str) {
            Ok(mapping) => {
                self.mappings.push_back(mapping);
                self.other = Some(&self.mappings.len() - 1);
                return Ok(());
            }
            Err(e) => {
                eprintln!("{:?}", e);
                Err(e)
            }
        }
    }

    pub fn is_aligned(&self) -> bool {
        self.first.is_some() && self.other.is_some()
    }

    pub fn _desync(&mut self) {
        *self = Self::new();
    }

    pub fn get_first_page_lines(&self) -> usize {
        let fold = self
            .mappings
            .iter()
            .filter(|m| m.is_first)
            .fold((0, 0), |(sum, count), value| {
                (value.page_lines + sum, count + 1)
            });
        (fold.0 as f64 / fold.1 as f64).round() as usize
    }

    pub fn get_other_page_lines(&self) -> usize {
        let fold = self
            .mappings
            .iter()
            .filter(|m| !m.is_first)
            .fold((0, 0), |(sum, count), value| {
                (value.page_lines + sum, count + 1)
            });
        (fold.0 as f64 / fold.1 as f64).round() as usize
    }

    pub fn get_mapping(&self, id: usize) -> Option<Mapping> {
        if id < self.mappings.len() {
            return Some(self.mappings[id].clone());
        }
        None
    }

    pub fn get_avg_ch(&self) -> f64 {
        let mut sum = 0;
        let mut count = 0;
        for map in self.mappings.iter() {
            sum += map.tot_chars;
            count += map.full_lines;
        }
        sum as f64 / count as f64
    }
}

pub(crate) fn find_ch(str: String, chs: Vector<Chapter>) -> Option<(usize, usize)> {
    for (index, ch) in chs.iter().enumerate() {
        let plain_text = xml_to_text(&ch.xml).replace("\n", " ").replace(".", " ");
        let p_clone = plain_text.clone();
        let t_clone = str
            .clone()
            .replace("-\n", "")
            .replace("\n", " ")
            .replace(".", " ");
        if let Some(offset) =
            OcrAlgorithms::fuzzy_match(p_clone, t_clone, OcrAlgorithms::fuzzy_linear_compare)
        {
            return Some((index, offset));
        }
    }
    None
}
