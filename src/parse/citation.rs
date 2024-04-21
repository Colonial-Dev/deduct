use std::cmp::PartialEq;
use std::fmt::Display;
use std::ops::RangeInclusive;

use once_cell::sync::Lazy;
use regex::Regex;

use super::normalize_ops;
use super::ParseError;
use super::LineRange;

#[derive(Debug, PartialEq, Eq)]
pub enum LineNumber {
    One(u16),
    Many(LineRange)
}

impl LineNumber {
    pub fn parse(i: &str) -> Result<Self, ParseError> {
        static NUM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\d{1,}"#).unwrap() );

        let i = i.trim();

        let extract = |iter: &mut regex::Matches| -> Result<u16, _> {
            let v = iter
                .next()
                .expect("Regex should have a capture")
                .as_str();

            let Ok(v) = v.parse::<u16>() else {
                return Err(ParseError::OversizeValue)
            };

            Ok(v)
        };

        let mut matches = NUM_REGEX.find_iter(i);

        match NUM_REGEX.find_iter(i).count() {
            0 | 3.. => Err(ParseError::BadLineNumber),
            1 => {
                Ok(Self::One(
                    extract(&mut matches)?
                ))
            }
            2 => {
                let s = extract(&mut matches)?;
                let e = extract(&mut matches)?;
                
                if e <= s {
                    return Err(ParseError::BadLineRange)
                }

                Ok( Self::Many(s..=e) )     
            },
        }
    }

    pub fn as_one(&self) -> u16 {
        let Self::One(v) = self else {
            panic!("Tried to interpret a line range as a single line")
        };

        *v
    }

    pub fn as_many(&self) -> RangeInclusive<u16> {
        let Self::Many(r) = self else {
            panic!("Tried to interpret a single line as a line range")
        };

        r.clone()
    }
}

impl Display for LineNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One(n)  => write!(f, "{n}")?,
            Self::Many(r) => write!(f, "{}-{}", r.start(), r.end())?
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum LineNumberType {
    One,
    Many
}

impl PartialEq<LineNumber> for LineNumberType {
    fn eq(&self, other: &LineNumber) -> bool {
        use LineNumber as LN;
        use LineNumberType as LT;

        matches!(
            (self, other),
            (LT::One, LN::One(_)) | (LT::Many, LN::Many(_))
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Citation {
    pub r: String,
    pub l: Vec<LineNumber>,
}

impl Citation {
    pub fn parse(i: &str) -> Result<Self, ParseError> {        
        static SEP_REGEX : Lazy<Regex> = Lazy::new(|| Regex::new(r#"[;,\s]+"#).unwrap() );
        
        if i.trim().is_empty() {
            return Err(ParseError::EmptyCitation)
        }

        let i = i.trim();

        let i = SEP_REGEX
            .replace_all(&normalize_ops(i), ",")
            .trim()
            .to_owned();

        let mut pieces = i.split(',').peekable();

        let Some(rule) = pieces.next() else {
            return Err(ParseError::MissingRule)
        };

        if pieces.peek().is_none() {
            return Ok(Self {
                r: rule.trim().to_owned(),
                l: Vec::new()
            })
        }

        let lines: Vec<_> = pieces
            .map(LineNumber::parse)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            r: rule.trim().to_owned(),
            l: lines,
        })
    }
}

impl Display for Citation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.r)?;

        for line in &self.l {
            write!(f, "{line} ")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let citation = Citation::parse("R4 1, 2-3 4").unwrap();

        assert_eq!(
            citation,
            Citation {
                r: String::from("R4"),
                l: vec![
                    LineNumber::One(1),
                    LineNumber::Many(2..=3),
                    LineNumber::One(4)
                ]
            }
        )
    }
}