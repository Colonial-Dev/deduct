use std::ops::RangeInclusive;

use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;

mod citation;
mod sentence;

pub mod consts {
    pub const NEG: &str = "¬";
    pub const CON: &str = "∧";
    pub const DIS: &str = "∨";
    pub const BIC: &str = "↔";
    pub const IMP: &str = "→";
    pub const BOT: &str = "⊥";
    pub const NEC: &str = "□";
    pub const POS: &str = "◇";
}

pub use sentence::Sentence;
pub use citation::{Citation, LineNumber, LineNumberType};

pub type LineRange   = RangeInclusive<u16>;
pub type ParseErrors = Vec<(u16, ParseError)>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("empty sentence")]
    EmptySentence,
    #[error("unbalanced parentheses")]
    UnbalancedParentheses,
    #[error("encountered invalid character(s) {0:?}")]
    InvalidCharacter(Vec<String>),
    #[error("too many operators or too few parentheses to disambiguate")]
    Ambiguous,
    #[error("missing connective/operator or misplaced parentheses")]
    MissingOp,
    #[error("misuse of unary operator internally in sentence")]
    BadUnary,
    #[error("misuse of contradiction symbol internally in sentence")]
    BadContradiction,
    #[error("empty citation")]
    EmptyCitation,
    #[error("citation does not cite a rule")]
    MissingRule,
    #[error("malformed line number")]
    BadLineNumber,
    #[error("cited a line range with the end at or before the start")]
    BadLineRange,
    #[error("line number too large")]
    OversizeValue,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Line {
    pub s: Sentence,
    pub c: Citation,
    pub n: u16,
    pub d: u16,
}

impl Line {
    pub fn cited_lines(&self) -> &[LineNumber] {
        self.c.l.as_slice()
    }

    pub fn is_premise(&self) -> bool {
        self.c.r == "PR"
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Proof {
    pub lines: Vec<Line>,
}

impl Proof {
    pub fn parse<'a, I>(i: I) -> Result<Self, ParseErrors> 
    where
        I: AsRef<[(u16, &'a str, &'a str)]>
    {
        let i = i.as_ref();

        let mut lines = vec![];
        let mut error = vec![];

        for (i, l) in i
            .iter()
            .enumerate()
            .map(|(i, l)| (i + 1, l) )
        {
            let (depth, sentence, citation) = l;

            let s = Sentence::parse(sentence);
            let c = Citation::parse(citation);

            if s.is_ok() && c.is_ok() {
                // Can't do multiple if lets in one line (yet)
                #[allow(clippy::unnecessary_unwrap)]
                lines.push(Line {
                    s: s.unwrap(),
                    c: c.unwrap(),
                    n: i as u16,
                    d: *depth,
                })
            } else {
                if let Err(e) = s {
                    error.push( (i as u16, e) )
                };

                if let Err(e) = c {
                    error.push( (i as u16, e) );
                }
            }
        }

        if !error.is_empty() {
            return Err(error);
        }

        Ok(Self { lines })
    }
    
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, n: u16) -> Option<&Line> {
        self.lines.get(n as usize - 1)
    }
}

/// Normalize operator shorthands in a given string.
pub fn normalize_ops(i: &str) -> String {
    use std::ops::Deref;
    use consts::*;
    
    static BIC_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:≡|<\->|<>)"#).unwrap(), BIC) );
    static IMP_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:⇒|⊃|->|>)"#).unwrap(), IMP) );
    static CON_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:\^|&|\.|·|\*)"#).unwrap(), CON) );
    static DIS_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"v"#).unwrap(), DIS) );
    static NEG_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:~|∼|-|−)"#).unwrap(), NEG) );
    static BOT_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:XX|#)"#).unwrap(), BOT) );
    static NEC_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"\[\]"#).unwrap(), NEC) );
    static POS_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"<>"#).unwrap(), POS) );
        
    let pairs = [
        BIC_REGEX.deref(),
        IMP_REGEX.deref(),
        CON_REGEX.deref(),
        DIS_REGEX.deref(),
        NEG_REGEX.deref(),
        BOT_REGEX.deref(),
        NEC_REGEX.deref(),
        POS_REGEX.deref(),
    ];

    let mut out = i.to_owned();

    for (regex, norm) in pairs {
        out = regex.replace_all(&out, *norm).to_string();
    }

    out
}