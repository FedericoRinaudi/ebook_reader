use std::error::Error;
use unicode_segmentation::UnicodeSegmentation;
use regex::Regex;
extern crate regex;
use druid::{im::Vector, Data, Lens};
use crate::algorithms::OcrAlgorithms;
use crate::book::chapter::Chapter;
use crate::utilities::xml_to_text;

#[derive(Default, Clone, Data, Lens, Debug, PartialEq)]
pub struct Mapping {
    pub page: usize, //Page number
    pub(crate) is_first: bool, //E' la prima pagina di un capitolo?
    pub(crate) tot_chars: usize, //Total sum of characters to get the average
    pub(crate) full_lines: usize, //Number of lines considered to get average characters
    pub(crate) text: String
}

impl Mapping {
    pub fn new(str:String, is_first: bool) -> Result<(Self, usize), Box<dyn Error>> {

        let mut init = Mapping {
            page: 0,
            is_first,
            tot_chars: 0,
            full_lines: 0,
            text: str
        };
        let line = init.page_stats()?;
        println!("{:?}", init);
        Ok((init,line))
    }

    fn page_stats(&mut self) -> Result<usize, Box<dyn Error>> {

        //ARRAY CON UNA LINEA PER OGNI RIGA
        let lines = self.text
            .split("\n")
            .filter(|s| {
                if s.graphemes(true).count() < 4 {
                    if let Some(pg_num) = Regex::new(r"\d+").unwrap().captures(s){
                        self.page = pg_num[0].parse::<usize>().unwrap();
                        println!("Found page number: {}", self.page)
                    }
                    return false
                }else {
                    return true
                }
            })
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        //Lines adesso contiene solo le linee "vere" ossia con almeno 4 caratteri

        let num_lines = lines.len(); //Salviamo il numero di linee trovate nella pagina

        let fold_res = lines
            .iter()
            .filter(|s| {
                let last = s.chars().last().unwrap();
                last.is_alphabetic() || last == '-'
            })
            .map(|s| s.len())
            .fold((0, 0), |(sum, count), value| {
                (sum + value, count + 1)
            });

        self.full_lines = fold_res.1;
        self.tot_chars = fold_res.0;
        Ok(num_lines)
    }

}

#[derive(Default, Clone, Data, Lens, PartialEq, Debug)]
pub struct OcrData {
    pub mappings: Vector<Mapping>,
    lines: (usize, usize)
}

impl OcrData {

    pub fn new() -> Self {
        OcrData {
            mappings: Vector::new(),
            lines: (0, 0)
        }
    }

    pub fn ocr_log(&mut self, str:String, is_first:bool) -> Result<usize, Box<dyn Error>> {
        match Mapping::new(str, is_first) {
            Ok((mapping,lines)) => {
                // Se questa è la prima volta che inseriamo una pagina iniziale/altra salviamo il num di riga
                if self.lines.0 == 0 && mapping.is_first {
                    self.lines.0 = lines
                } else if self.lines.1 == 0 && !mapping.is_first {
                    self.lines.1 = lines
                }

                self.mappings.push_back(mapping);

                return Ok(self.mappings.len() -1)
        }
            Err(e) => {
                eprintln!("{:?}", e);
                Err(e)
            }
        }
    }

    pub fn get_mapping(&self, id:usize) -> Option<Mapping>{
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
        println!("AVERAGE CHARS PER LINE: {}\n", sum as f64/count as f64);
        return sum as f64/count as f64
    }

    pub fn get_lines(&self) -> (usize, usize) {
        self.lines
    }

}


pub(crate) fn find_ch(str:String, chs: Vector<Chapter>) -> Option<(usize, usize)> {
    for (index, ch) in chs.iter().enumerate() {
        let plain_text = xml_to_text(&ch.xml).replace("\n", " ").replace(".", " ");
        let p_clone = plain_text.clone();
        let t_clone = str.clone()
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