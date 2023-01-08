use std::collections::HashMap;

pub struct OcrAlgorithms {}

impl OcrAlgorithms {
    pub fn _fuzzy_freq_compare(v1: &[String], v2: &[String], tol: f64) -> bool {
        let mut map1 = HashMap::new();
        let mut map2 = HashMap::new();

        for word in v1 {
            *map1.entry(word).or_insert(0) += 1;
        }

        for word in v2 {
            *map2.entry(word).or_insert(0) += 1;
        }

        let mut common_words = 0;
        let mut total_words = 0;

        for (word, count) in map1.iter() {
            total_words += count;

            if let Some(count2) = map2.get(word) {
                common_words += if count < count2 { count } else { count2 };
            }
        }

        (common_words as f64 / total_words as f64) >= tol
    }

    pub fn fuzzy_match(
        chapter: String,
        page: String,
        algorithm: fn(&[String], &[String], f64) -> bool,
    ) -> Option<usize> {
        let chapter: Vec<String> = chapter
            .split(" ")
            .map(|el| el.chars().filter(|c| c.is_alphabetic()).collect::<String>())
            .filter(|w| w.len() >= 5)
            .collect();
        let page: Vec<String> = page
            .split(" ")
            .map(|el| el.chars().filter(|c| c.is_alphabetic()).collect::<String>())
            .filter(|w| w.len() >= 5)
            .collect();
        let mut offset = 0;
        if page.len() == 0 {
            return None;
        };
        for w in chapter.windows(page.len()) {
            if algorithm(w, &page, 0.5) {
                return Some(offset);
            }
            offset += 1
        }
        None
    }

    pub fn fuzzy_linear_compare(a: &[String], b: &[String], tol: f64) -> bool {
        let mut eq: usize = 0;

        for (i, e) in a.iter().enumerate() {
            if *e == *(b.get(i).unwrap_or(&String::new())) {
                eq += 1;
            }
        }
        (eq as f64 / a.len() as f64) >= tol
    }
}
