use once_cell::sync::Lazy;
use regex::Regex;

use super::normalize_ops;
use super::ParseError;
use super::consts::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Sentence {
    /// An atomic predicate (A-Z, capitals only.)
    Atomic(char),
    /// A "signal" operator (lone contradiction or necessity.)
    Signal(char),
    /// Negation.
    Neg(Box<Self>),
    /// Necessity.
    Nec(Box<Self>),
    /// Possibility.
    Pos(Box<Self>),
    /// Conjunction.
    Con(Box<Self>, Box<Self>),
    /// Disjunction.
    Dis(Box<Self>, Box<Self>),
    /// Conditional implication.
    Imp(Box<Self>, Box<Self>),
    /// Biconditional implication.
    Bic(Box<Self>, Box<Self>),
}

impl Sentence {
    pub fn parse(i: &str) -> Result<Self, ParseError> {
        static SIGNAL_REGEX   : Lazy<Regex> = Lazy::new(|| Regex::new("^[⊥□]$").unwrap() );
        static BOT_REGEX      : Lazy<Regex> = Lazy::new(|| Regex::new("⊥").unwrap() );
        static ATOMIC_REGEX   : Lazy<Regex> = Lazy::new(|| Regex::new("^[A-Z]$").unwrap() );
        static OP_REGEX       : Lazy<Regex> = Lazy::new(|| Regex::new("[¬∧∨↔→⊥□◇]").unwrap() );
        
        // Take care of any loose whitespace before we proceed
        let i = i.trim();

        // Emptiness check
        if i.is_empty() {
            return Err(ParseError::EmptySentence)
        }

        // Normalize parenthesis and operator shorthands (i.e. <-> becomes ↔)
        let i = normalize_braces( &normalize_ops(i) );

        // Compute parenthesis depths
        let d = compute_depths(&i)?;

        // Remove redundant outer parentheses
        if d[0] == 1 {
            let mut m = true;

            for (n, _) in i
                .chars()
                .enumerate()
                .skip(1)
                .take(i.chars().count() - 2) 
            {
                m = m && d[n] > 0;
            }
            
            if m {
                let rest: String = i
                    .chars()
                    .skip(1)
                    .take(i.chars().count() - 2)
                    .collect();
                
                return Self::parse(&rest);
            }
        }

        // Check for any invalid characters that remain after normalization
        invalid_chars(&i)?;

        if SIGNAL_REGEX.is_match(&i) {
            let c = i.chars()
                .nth(0)
                .expect("Signal regular expection matched an empty string");

            return Ok( Self::Signal(c) )
        }

        if BOT_REGEX.is_match(&i) {
            return Err(ParseError::BadContradiction)
        }

        // No operators means we should be dealing with an atomic.
        if ATOMIC_REGEX.is_match(&i) {
            let c = i.chars()
                .nth(0)
                .expect("Atomic regular expection matched an empty string");
            
            return Ok( Self::Atomic(c) )
        }

        let mut main_op_c = None;
        let mut main_op_p = None;

        // Locate the main operator.
        for (n, c) in i.chars().enumerate() {
            if OP_REGEX.is_match( &c.to_string() ) && d[n] == 0 {
                match main_op_c {
                    None => {
                        main_op_c = Some(c);
                        main_op_p = Some(n);
                    },
                    Some(m) => {
                        if is_bin_op(m) && is_bin_op(c) {
                            return Err(ParseError::Ambiguous)
                        }
                        else if is_una_op(m) && is_bin_op(c) {
                            main_op_c = Some(c);
                            main_op_p = Some(n);
                        }
                    }
                }
            }
        }

        let Some(main_op_c) = main_op_c.map(String::from) else {
            return Err(ParseError::MissingOp)
        };

        let main_op_p = main_op_p.expect("Main operator position should be known");

        if matches!(main_op_c.as_str(), NEG | NEC | POS) {
            if main_op_p != 0 {
                return Err(ParseError::BadUnary)
            }

            let rest = i.chars().skip(1).collect::<String>();
            let rest = Box::new( Self::parse(&rest)? );

            return match main_op_c.as_str() {
                NEG => Ok( Self::Neg(rest) ),
                NEC => Ok( Self::Nec(rest) ),
                POS => Ok( Self::Pos(rest) ),
                _   => unreachable!("Tried to parse a non-existent main unary operator {main_op_c}")
            }
        }

        let l: String = i.chars().take(main_op_p).collect();
        let r: String = i.chars().skip(main_op_p + 1).collect();

        let l = Box::new( Self::parse(&l)? );
        let r = Box::new( Self::parse(&r)? );

        match main_op_c.as_str() {
            CON => Ok( Self::Con(l, r) ),
            DIS => Ok( Self::Dis(l, r) ),
            IMP => Ok( Self::Imp(l, r) ),
            BIC => Ok( Self::Bic(l, r) ),
            _   => unreachable!("Tried to parse a non-existent main binary operator {main_op_c}")
        }
    }

    pub fn negated(&self) -> Self {
        Self::Neg( self.clone().box_up() )
    }

    pub fn box_up(self) -> Box<Self> {
        Box::new(self)
    }
}

impl PartialEq<&Box<Sentence>> for Sentence {
    fn eq(&self, other: &&Box<Sentence>) -> bool {
        *self == ***other
    }
}

impl PartialEq<&Box<Sentence>> for &Sentence {
    fn eq(&self, other: &&Box<Sentence>) -> bool {
        **self == ***other
    }
}

impl PartialEq<Sentence> for &Box<Sentence> {
    fn eq(&self, other: &Sentence) -> bool {
        ***self == *other
    }
}

impl PartialEq<&Sentence> for &Box<Sentence> {
    fn eq(&self, other: &&Sentence) -> bool {
        ***self == **other
    }
}

/// Normalize braces.
fn normalize_braces(i: &str) -> String {
    use std::ops::Deref;
    
    static BRO_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:\[|\{)"#).unwrap(), "(") );
    static BRC_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:\]|\})"#).unwrap(), ")") );
    
    let pairs = [
        BRO_REGEX.deref(),
        BRC_REGEX.deref()
    ];

    let mut out = i.to_owned();

    for (regex, norm) in pairs {
        out = regex.replace_all(&out, *norm).to_string();
    }

    out
}

fn invalid_chars(i: &str) -> Result<(), ParseError> {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[^A-Z¬∨∧↔→⊥□◇\s\)\(\]\[\}\{]"#).unwrap() );

    let captures: Vec<_> = REGEX.find_iter(i)
        .map(|m| m.as_str() )
        .map(|s| s.to_owned() )
        .collect();

    if !captures.is_empty() {
        return Err( ParseError::InvalidCharacter(captures) )
    }

    Ok(())
}

fn compute_depths(i: &str) -> Result<Box<[u16]>, ParseError> {
    let mut c_depth = 0_u16;
    let mut v_depth = vec![];

    for c in i.chars() {
        match c {
            '(' => c_depth = c_depth.saturating_add(1),
            ')' => c_depth = c_depth.saturating_sub(1),
            _   => ()
        }

        v_depth.push(c_depth);
    }

    if c_depth != 0 {
        return Err(ParseError::UnbalancedParentheses)
    }

    Ok( v_depth.into_boxed_slice() )
}

fn is_una_op(c: char) -> bool {
    matches!(c, '¬' | '⊥' | '□' | '◇')
}

fn is_bin_op(c: char) -> bool {
    matches!(c, '∧'| '∨' | '↔' | '→')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_sentence() {
        assert_eq!(
            Sentence::parse("").unwrap_err(),
            ParseError::EmptySentence,
        );
    }

    #[test]
    fn unbalanced() {
        assert_eq!(
            Sentence::parse("(A ^ B").unwrap_err(),
            ParseError::UnbalancedParentheses
        );
    }

    #[test]
    fn invalid_char() {
        assert_eq!(
            Sentence::parse("(Aa ^ Bb)!").unwrap_err(),
            ParseError::InvalidCharacter(vec!["a".to_owned(), "b".to_owned(), "!".to_owned()])
        );
    }

    #[test]
    fn ambiguity() {
        assert_eq!(
            Sentence::parse("A ^^ B").unwrap_err(),
            ParseError::Ambiguous
        );
    }

    #[test]
    fn missing_op() {
        assert_eq!(
            Sentence::parse("A B").unwrap_err(),
            ParseError::MissingOp
        );
    }

    // [¬∧∨↔→⊥□◇]
    #[test]
    fn bad_unary() {
        assert_eq!(
            Sentence::parse("A¬B").unwrap_err(),
            ParseError::BadUnary
        );

        assert_eq!(
            Sentence::parse("A□B").unwrap_err(),
            ParseError::BadUnary
        );

        assert_eq!(
            Sentence::parse("A◇B").unwrap_err(),
            ParseError::BadUnary
        );
    }

    #[test]
    fn atomic() {
        let s = Sentence::parse("A").unwrap();

        assert_eq!(
            s,
            Sentence::Atomic('A')
        );
    }

    #[test]
    fn signal() {
        let bot = Sentence::parse("#").unwrap();
        let pos = Sentence::parse("[]").unwrap();

        assert_eq!(
            bot,
            Sentence::Signal('⊥')
        );

        assert_eq!(
            pos,
            Sentence::Signal('□')
        );
    }
    
    #[test]
    fn neg() {
        let neg = Sentence::parse("~A").unwrap();

        assert_eq!(
            neg,
            Sentence::Neg(
                Sentence::Atomic('A').box_up()
            )
        );

        let neg = Sentence::parse("~~A").unwrap();

        assert_eq!(
            neg,
            Sentence::Neg(
                Sentence::Neg(
                    Sentence::Atomic('A').box_up()
                ).box_up()
            )
        );
    }

    #[test]
    fn nec() {
        let nec = Sentence::parse("□A").unwrap();

        assert_eq!(
            nec,
            Sentence::Nec(
                Sentence::Atomic('A').box_up()
            )
        );

        let nec = Sentence::parse("□□A").unwrap();

        assert_eq!(
            nec,
            Sentence::Nec(
                Sentence::Nec(
                    Sentence::Atomic('A').box_up()
                ).box_up()
            )
        );   
    }

    #[test]
    fn pos() {
        let pos = Sentence::parse("◇A").unwrap();

        assert_eq!(
            pos,
            Sentence::Pos(
                Sentence::Atomic('A').box_up()
            )
        );

        let pos = Sentence::parse("◇◇A").unwrap();

        assert_eq!(
            pos,
            Sentence::Pos(
                Sentence::Pos(
                    Sentence::Atomic('A').box_up()
                ).box_up()
            )
        );   
    }

    #[test]
    fn con() {
        let con = Sentence::parse("A ^ B").unwrap();

        assert_eq!(
            con,
            Sentence::Con(
                Sentence::Atomic('A').box_up(),
                Sentence::Atomic('B').box_up()
            )
        );
    }

    #[test]
    fn dis() {
        let dis = Sentence::parse("A v B").unwrap();

        assert_eq!(
            dis,
            Sentence::Dis(
                Sentence::Atomic('A').box_up(),
                Sentence::Atomic('B').box_up()
            )
        );
    }

    #[test]
    fn imp() {
        let imp = Sentence::parse("A -> B").unwrap();

        assert_eq!(
            imp,
            Sentence::Imp(
                Sentence::Atomic('A').box_up(),
                Sentence::Atomic('B').box_up()
            )
        );
    }

    #[test]
    fn bic() {
        let bic = Sentence::parse("A <-> B").unwrap();

        assert_eq!(
            bic,
            Sentence::Bic(
                Sentence::Atomic('A').box_up(),
                Sentence::Atomic('B').box_up()
            )
        );
    }
}