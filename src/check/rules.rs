use thiserror::Error;

use crate::parse::*;

const BOT: char = '⊥';
const NEC: char = '□';

pub type CheckErrors = Vec<(u16, CheckError)>;

pub const TFL_BASIC: &[(&str, &dyn Rule)] = &[
    ("R", &Reiteration),
    ("∧I", &ConjunctionIntr),
    ("∧E", &ConjunctionElim),
    ("∨I", &DisjunctionIntr),
    ("∨E", &DisjunctionElim),
    ("→I", &ConditionalIntr),
    ("→E", &ConditionalElim),
    ("↔I", &BiconditionalIntr),
    ("↔E", &BiconditionalElim),
    ("¬I", &NegationIntr),
    ("¬E", &NegationElim),
    ("IP", &IndirectProof),
    ("X", &Explosion),
];

// DERIVED
// - BASIC +
// - DS (Disjunctive syllogism)
// - MT (Modus tollens)
// - DNE
// - LEM
// - DeM
// SYSTEM_K
// - BASIC + DERIVED
// - Necessity I/E
// - Possibility definition
// - Modal conversion
// SYSTEM_T
// - SYSTEM_K +
// - RT
// SYSTEM_S4
// - SYSTEM_T +
// - R4
// SYSTEM_S5
// - SYSTEM_S4 +
// - R5
// PREMISE (DEDUCT_INTERNAL_PREMISE_DO_NOT_USE)
// - Special, internal rule injected to qualify premises.
// - Cannot be referenced by the user.
// PLACEHOLDER
// - Special, non-standard rule. Similar to a premise, but usable anywhere and always validates the cited line.
// - Can be used to silence the checker when working on a different part of the proof.
// - Checker will point out a proof that is valid but still contains placeholder citations.
pub trait Rule {
    /// Returns the order and type of lines uses of this rule should cite.
    fn line_ord(&self) -> &[LineNumberType];
    /// Verifies that the rule cited is used correctly.
    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError>;

    /// Validate the use of this rule in justifying the provided line.
    fn validate(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        if self.line_ord().len() != l.cited_lines().len() {
            return Err(CheckError::BadLineCount)
        }

        // Ensure expected line number types match the actual types.
        if self
            .line_ord()
            .iter()
            .zip( l.cited_lines() )
            .any(|(e, a)| e != a) 
        {
            return Err(CheckError::BadLineType)
        }

        // Ensure we are not citing ourselves or the future.
        // This also captures lines that do not exist.
        if l
            .cited_lines()
            .iter()
            .any(|ln| match ln {
                // Single line citations must be at least one,
                // and cannot refer to current or future lines.
                LineNumber::One(n) => *n >= l.n || *n < 1,
                // The start of a line range must be at least 1, and the end must be at least 2.
                // The end of the line range must not be a current or future line.
                LineNumber::Many(r) => {
                    *r.start() < 1 || *r.end() < 2 || *r.end() >= l.n
                }
            })
        {
            return Err(CheckError::BadCitation)
        }

        // Ensure all line ranges are citing a valid, complete subproof.
        if l
            .cited_lines()
            .iter()
            .filter_map(|ln| match ln {
                LineNumber::Many(r) => Some(r),
                _ => None
            })
            .any(|r| {
                let s = r.start();
                let e = r.end();

                // These unwraps are safe, as non-existent lines
                // would have triggered an error in the previous scan.
                let sd = p.line(*s).map(|l| l.d).unwrap();
                let ed = p.line(*e).map(|l| l.d).unwrap();

                // If the start or end depths are zero, then this can't be a subproof.
                // If the start and end have different depths, then the range is oversized.
                // If the end is greater than or equal to the current line, then the subproof has not been closed.
                if (sd < 1 || ed < 1) || (sd != ed) || (sd >= l.n) {
                    return true;
                }
                
                // If any line within the bounds of the alleged subproof
                // has a depth *less than* that of the start line, then
                // the range is oversized.
                for n in r.clone() {
                    if p.line(n).map(|l| l.d).unwrap() < sd {
                        return true;
                    }
                }

                // If the line after the end doesn't have a lower depth,
                // then the subproof has not been closed.
                let next = p.line(*e + 1).unwrap();

                if next.d >= ed && !next.is_premise() {
                    return true;
                }

                false
            })
        {
            return Err(CheckError::BadRange)
        }

        // Accessibility index for the line being validated.
        let mut access_idx = vec![false; p.len()];

        // Precompute accessibility relative to all previous lines in the proof.
        // (Future lines are by definition inaccessible.)
        //
        // The ceiling value is initialized to the depth of the current line.
        let mut ceil = l.d;

        // Step backwards through the proof from the current line.
        for n in (l.n..=1).rev() {
            let d = p.line(n).map(|l| l.d).unwrap();

            #[allow(clippy::comparison_chain)]
            // If the line's depth is equal to the ceiling value, it is reachable.
            if d == ceil {
                access_idx[n as usize - 1] = true;
            }
            // If the line is shallower than the ceiling value, it is reachable,
            // but the ceiling is lowered to match.
            else if d < ceil {
                access_idx[n as usize - 1] = true;
                ceil -= 1;
            } 
            // If the line is greater than the ceiling value, it is not reachable.
            else {
                access_idx[n as usize - 1] = false;
            }
        }

        // Ensure that no unavailable lines or subproofs are being cited.
        if l
            .cited_lines()
            .iter()
            .any(|ln| match ln {
                LineNumber::One(n) => {
                    access_idx[*n as usize - 1]
                }
                LineNumber::Many(r) => {
                    // We only need to check the start depth,
                    // as subproof range validity was checked in the previous scan.
                    access_idx[*r.start() as usize - 1]
                }
            })
        {
            return Err(CheckError::Unavailable)
        }

        self.is_right(p, l)?;

        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CheckError {
    #[error("cited a rule that does not exist or is badly formed")]
    NoSuchRule,
    #[error("cited too few or too many lines for the specified rule")]
    BadLineCount,
    #[error("cited a line range where a single line was expected (or vice versa)")]
    BadLineType,
    #[error("cited a rule that was used incorrectly")]
    BadUsage,
    #[error("cited a current or future line, or a line that does not exist")]
    BadCitation,
    #[error("cited a line range that does not correspond to a subproof")]
    BadRange,
    #[error("cited an unavailable line or subproof")]
    Unavailable,
}

/* enum Citations<'a> {
    One(&'a Sentence),
    Many(&'a Sentence, &'a Sentence)
} */

fn cited_sentence<'a>(p: &'a Proof, l: &Line, n: usize) -> Result<&'a Sentence, CheckError> {
    let Some(l) = p.line( l.cited_lines()[n].as_one() ) else {
        return Err(CheckError::BadCitation)
    };

    Ok(&l.s)
}

fn cited_subproof<'a>(p: &'a Proof, l: &Line, n: usize) -> Result<(&'a Sentence, &'a Sentence), CheckError> {
    let range = l.cited_lines()[n].as_many();

    Ok((
        &p.line( *range.start() ).unwrap().s,
        &p.line( *range.end() ).unwrap().s
    ))
}

pub struct Premise;

impl Rule for Premise {
    fn line_ord(&self) -> &[LineNumberType] {
        &[]
    }

    fn is_right(&self, _p: &Proof, _l: &Line) -> Result<(), CheckError> {
        Ok(())
    }
}

struct Reiteration;

impl Rule for Reiteration {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {        
        let source = cited_sentence(p, l, 0)?;

        if source != &l.s {
            return Err(CheckError::BadUsage)
        }

        Ok(())
    }
}

struct ConjunctionIntr;

impl Rule for ConjunctionIntr {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One, LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let s_a = cited_sentence(p, l, 0)?;
        let s_b = cited_sentence(p, l, 1)?;

        let Sentence::Con(lhs, rhs) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if (**lhs == *s_a || **lhs == *s_b) && (**rhs == *s_a || **rhs == *s_b) {
            Ok(())
        } else {
            Err(CheckError::BadUsage)
        }
    }
}

struct ConjunctionElim;

impl Rule for ConjunctionElim {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let source = cited_sentence(p, l, 0)?;

        let Sentence::Con(lhs, rhs) = source else {
            return Err(CheckError::BadUsage)
        };

        match (**lhs == l.s, **rhs == l.s) {
            (true, _) => Ok(()),
            (_, true) => Ok(()),
            _ => Err(CheckError::BadUsage)
        }
    }
}

struct DisjunctionIntr;

impl Rule for DisjunctionIntr {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let source = cited_sentence(p, l, 0)?;

        let Sentence::Dis(lhs, rhs) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if (**lhs == *source) || (**rhs == *source) {
            Ok(())
        } else {
            Err(CheckError::BadUsage)
        }
    }
}

struct DisjunctionElim;

impl Rule for DisjunctionElim {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One, LineNumberType::Many, LineNumberType::Many]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let source = cited_sentence(p, l, 0)?;

        let Sentence::Dis(lhs, rhs) = source else {
            return Err(CheckError::BadUsage)
        };

        let (p_1, c_1) = cited_subproof(p, l, 1)?;
        let (p_2, c_2) = cited_subproof(p, l, 2)?;

        if (*c_1 != l.s) || (*c_2 != l.s) {
            return Err(CheckError::BadUsage)
        }

        if (*p_1 == **lhs && *p_2 == **rhs) || (*p_1 == **rhs && *p_2 == **lhs) {
            Ok(())
        } else {
            Err(CheckError::BadUsage)
        }
    }
}

struct ConditionalIntr;

impl Rule for ConditionalIntr {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::Many]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let (p, c) = cited_subproof(p, l, 0)?;

        let Sentence::Imp(lhs, rhs) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if **lhs == *p && **rhs == *c {
            Ok(())
        } else {
            Err(CheckError::BadUsage)
        }
    }
}

struct ConditionalElim;

impl Rule for ConditionalElim {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One, LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let s_1 = cited_sentence(p, l, 0)?;
        let s_2 = cited_sentence(p, l, 1)?;
        
        if let Sentence::Imp(lhs, rhs) = s_1 {
            if **lhs == *s_2 && **rhs == l.s {
                return Ok(())
            }
        }

        if let Sentence::Imp(lhs, rhs) = s_2 {
            if **lhs == *s_1 && **rhs == l.s {
                return Ok(())
            }
        }

        Err(CheckError::BadUsage)
    }
}

struct BiconditionalIntr;

impl Rule for BiconditionalIntr {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::Many, LineNumberType::Many]
    }
    
    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let (p_1, c_1) = cited_subproof(p, l, 0)?;
        let (p_2, c_2) = cited_subproof(p, l, 1)?;

        let Sentence::Bic(lhs, rhs) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if (**lhs == *p_1 && **rhs == *p_2) && (**lhs == *c_2 && **rhs == *c_1) {
            return Ok(())
        }

        if (**lhs == *p_2 && **rhs == *p_1) && (**lhs == *c_1 && **rhs == *c_2) {
            return Ok(())
        }

        Err(CheckError::BadUsage)
    }
}

struct BiconditionalElim;

impl Rule for BiconditionalElim {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One, LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let s_1 = cited_sentence(p, l, 0)?;
        let s_2 = cited_sentence(p, l, 1)?;

        let Sentence::Bic(lhs, rhs) = s_1 else {
            return Err(CheckError::BadUsage)
        };

        if (**lhs == *s_2 && **rhs == l.s) || (**rhs == *s_2 && **lhs == l.s) {
            return Ok(())
        }

        Err(CheckError::BadUsage)
    }
}

struct NegationIntr;

impl Rule for NegationIntr {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::Many]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let (p, c) = cited_subproof(p, l, 0)?;

        let Sentence::Signal(BOT) = c else {
            return Err(CheckError::BadUsage)
        };

        if let Sentence::Neg(s) = &l.s {
            if **s == *p {
                return Ok(())
            }
        }

        Err(CheckError::BadUsage)
    }
}

struct NegationElim;

impl Rule for NegationElim {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One, LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let s_1 = cited_sentence(p, l, 0)?;
        let s_2 = cited_sentence(p, l, 1)?;

        let Sentence::Signal(BOT) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if let Sentence::Neg(s_1) = s_1 {
            if **s_1 == *s_2 {
                return Ok(())
            } else {
                return Err(CheckError::BadUsage)
            }
        }

        if let Sentence::Neg(s_2) = s_2 {
            if **s_2 == *s_1 {
                return Ok(())
            } else {
                return Err(CheckError::BadUsage)
            }
        }

        Err(CheckError::BadUsage)
    }
}

struct Explosion;

impl Rule for Explosion {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }
    
    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let source = cited_sentence(p, l, 0)?;

        let Sentence::Signal(BOT) = source else {
            return Err(CheckError::BadUsage)
        };

        Ok(())
    }
}

struct IndirectProof;

impl Rule for IndirectProof {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::Many]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let (p, c) = cited_subproof(p, l, 0)?;

        let Sentence::Neg(p) = p else {
            return Err(CheckError::BadUsage)
        };

        let Sentence::Signal(BOT) = c else {
            return Err(CheckError::BadUsage)
        };

        if **p != l.s {
            return Err(CheckError::BadUsage)
        }

        Ok(())
    }
}