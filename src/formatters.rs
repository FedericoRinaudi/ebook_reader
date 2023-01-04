use druid::text::{Formatter, Selection, Validation, ValidationError};
use leptess::capi::Sel;

pub struct CustomFormatter {}
impl CustomFormatter {
    pub fn new() -> Self {
        Self {}
    }
}
impl Formatter<usize> for CustomFormatter {
    fn format(&self, value: &usize) -> String {
        return if *value == 0 as usize {
            String::new()
        } else {
            (*value).to_string()
        };
    }

    fn validate_partial_input(&self, input: &str, sel: &Selection) -> Validation {
        match input.parse::<usize>() {
            Ok(val) => Validation::success(),
            Err(e) => {
                if input == "" {
                    Validation::success()
                } else {
                    Validation::failure(e)
                }
            }
        }
    }

    fn value(&self, input: &str) -> Result<usize, ValidationError> {
        match input.parse::<usize>() {
            Ok(val) => Ok(val),
            Err(e) => {
                if input == "" {
                    Ok(0 as usize)
                } else {
                    Err(ValidationError::new(e))
                }
            }
        }
    }
}
