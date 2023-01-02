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
    pub page_lines: usize
}

impl Mapping {
    pub fn new(str:String) -> Result<Self, Box<dyn Error>> {

        let mut init = Mapping {
            page: 0,
            is_first: false,
            tot_chars: 0,
            full_lines: 0,
            page_lines: 0
        };
        init.page_stats(str)?;
        //println!("{:?}", init);
        Ok(init)
    }

    fn page_stats(&mut self, str:String) -> Result<(), Box<dyn Error>> {

        //ARRAY CON UNA LINEA PER OGNI RIGA
        let lines = str
            .split("\n")
            .filter(|s| {
                if s.graphemes(true).count() < 4 { //TODO: Riconosci intestazioni al posto di controllare solo numero caratteri, hint: usa funzione di spaziatura (?)
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
        // Lines adesso contiene solo le linee "vere" ossia con almeno 4 caratteri
        // TODO: COnsidera linee con meno di 4 caratteri potenzialmente valide

        self.page_lines = lines.len(); //Salviamo il numero di linee trovate nella pagina

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
        Ok(())
    }

}

#[derive(Default, Clone, Data, Lens, PartialEq, Debug)]
pub struct OcrData {
    pub mappings: Vector<Mapping>,
    pub first: Option<usize>,
    pub other: Option<usize>
}

impl OcrData {

    pub fn new() -> Self {
        OcrData {
            mappings: Vector::new(),
            first: None,
            other: None
        }
    }

    pub fn ocr_log(&mut self, str:String) -> Result<usize, ()> {
        if self.first.is_none() || self.other.is_none(){
            return Err(());
        }
        match Mapping::new(str) {
            Ok(mut mapping) => {
                let first_lines = self.mappings[self.first.unwrap()].page_lines;
                let other_lines = self.mappings[self.other.unwrap()].page_lines;
                let range =  first_lines-2..first_lines+3;
                let other_range =  other_lines-2..other_lines+3;
                if range.contains(&mapping.page_lines) {mapping.is_first = true }else if !other_range.contains(&mapping.page_lines) {return Err(())}
                self.mappings.push_back(mapping);

                return Ok(&self.mappings.len() -1)
        }
            Err(e) => {
                eprintln!("{:?}", e);
                Err(())
            }
        }
    }

    pub fn ocr_log_first(&mut self, str:String) -> Result<(), Box<dyn Error>> {
        match Mapping::new(str) {
            Ok(mut mapping) => {
                mapping.is_first = true;
                self.mappings.push_back(mapping);
                self.first = Some(&self.mappings.len() -1);
                return Ok(())
            }
            Err(e) => {
                eprintln!("{:?}", e);
                Err(e)
            }
        }
    }

    pub fn ocr_log_other(&mut self, str:String) -> Result<(), Box<dyn Error>> {
        match Mapping::new(str) {
            Ok(mapping) => {
                self.mappings.push_back(mapping);
                self.other = Some(&self.mappings.len() -1);
                return Ok(())
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

    pub fn get_first_page_lines(&self) -> usize {
        let fold = self.mappings.iter().filter(|m| m.is_first).fold((0, 0), |(sum, count), value| {
            (value.page_lines + sum, count + 1)});
        (fold.0 as f64/fold.1 as f64).ceil() as usize
    }

    pub fn get_other_page_lines(&self) -> usize {
        let fold = self.mappings.iter().filter(|m| !m.is_first).fold((0, 0), |(sum, count), value| {
            (value.page_lines + sum, count + 1)});
        (fold.0 as f64/fold.1 as f64).ceil() as usize
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