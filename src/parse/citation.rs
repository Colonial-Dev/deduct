use std::ops::RangeInclusive;

use once_cell::sync::Lazy;
use regex::Regex;

use super::normalize_ops;
use super::ParseError;

#[derive(Debug, PartialEq, Eq)]
pub enum LineNumber {
    One(u16),
    Many(RangeInclusive<u16>)
}

impl LineNumber {
    pub fn parse(i: &str) -> Result<Self, ParseError> {
        static NUM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\d{1,}"#).unwrap() );
        
        match NUM_REGEX.find_iter(i).count() {
            0 | 3.. => Err(ParseError::BadLineNumber),
            1 => {
                let v = NUM_REGEX
                    .find_iter(i)
                    .nth(0)
                    .expect("Regex should have one capture")
                    .as_str();

                let Ok(v) = v.parse::<u16>() else {
                    return Err(ParseError::OversizeValue)
                };
                
                Ok( Self::One(v) )
            }
            2 => {
                let l = NUM_REGEX
                    .find_iter(i)
                    .nth(0)
                    .expect("Regex should have one capture")
                    .as_str();

                let r = NUM_REGEX
                    .find_iter(i)
                    .nth(1)
                    .expect("Regex should have one capture")
                    .as_str();

                let Ok(l) = l.parse::<u16>() else {
                    return Err(ParseError::OversizeValue)
                };

                let Ok(r) = r.parse::<u16>() else {
                    return Err(ParseError::OversizeValue)
                };
                
                Ok( Self::Many(l..=r) )             
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Citation {
    pub r: String,
    pub l: Vec<LineNumber>,
}

impl Citation {
    pub fn parse(i: &str) -> Result<Self, ParseError> {        
        static SEP_REGEX : Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?:\s*,\s*|\s{1,})"#).unwrap() );
        
        let i = i.trim();

        if i.is_empty() {
            return Err(ParseError::EmptyCitation)
        }

        let i = normalize_ops(i);

        let r: String = i
            .chars()
            .take_while(|c| !c.is_ascii_digit() )
            .filter(|c| !c.is_whitespace() )
            .collect();

        let r = r.trim().to_owned();

        let l: String = i
            .chars()
            .skip_while(|c| !c.is_ascii_digit() )
            .collect();

        let l = SEP_REGEX
            .replace_all(&l, ",")
            .split(',')
            .map(LineNumber::parse)
            .try_fold(vec![], |mut acc, n| {
                acc.push(n?);
                Ok(acc)
            })?;

        Ok(Self{ r, l })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let citation = Citation::parse("~I 1, 2-3 4").unwrap();

        assert_eq!(
            citation,
            Citation {
                r: String::from("¬I"),
                l: vec![
                    LineNumber::One(1),
                    LineNumber::Many(2..=3),
                    LineNumber::One(4)
                ]
            }
        )
    }
}