use thiserror::Error;

use crate::parse::*;

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

pub const TFL_DERIVED: &[(&str, &dyn Rule)] = &[
    ("DS", &DisjunctiveSyllogism),
    ("MT", &ModusTollens),
    ("DNE", &Dne),
    ("LEM", &Lem),
    ("DeM", &DeMorgan),
    ("DEM", &DeMorgan),
];

pub const SYSTEM_K: &[(&str, &dyn Rule)] = &[
    ("□I", &NecessityIntr),
    ("□E", &NecessityElim),
    ("Def◇", &PossibilityDef),
    ("MC", &ModalConversion)
];

pub const SYSTEM_T: &[(&str, &dyn Rule)] = &[
    ("RT", &RT)
];

pub const SYSTEM_S4: &[(&str, &dyn Rule)] = &[
    ("R4", &R4)
];

pub const SYSTEM_S5: &[(&str, &dyn Rule)] = &[
    ("R5", &R5)
];
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
pub trait Rule {
    /// Returns the order and type of lines uses of this rule should cite.
    fn line_ord(&self) -> &[LineNumberType];
    /// Verifies that the rule cited is used correctly.
    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError>;

    /// Returns whether or not the rule is only usable in a strict subproof.
    /// 
    /// Defaults to `false`.
    fn strict_only(&self) -> bool {
        false
    }

    /// Validate the use of this rule in justifying the provided line.
    fn validate(&self, p: &Proof, line: &Line) -> Result<(), CheckError> {
        if self.line_ord().len() != line.cited_lines().len() {
            return Err(CheckError::BadLineCount)
        }

        // Ensure expected line number types match the actual types.
        if self
            .line_ord()
            .iter()
            .zip( line.cited_lines() )
            .any(|(e, a)| e != a) 
        {
            return Err(CheckError::BadLineType)
        }

        // Ensure we are not citing ourselves or the future.
        // This also captures lines that do not exist.
        if line
            .cited_lines()
            .iter()
            .any(|ln| match ln {
                // Single line citations must be at least one,
                // and cannot refer to current or future lines.
                LineNumber::One(n) => *n >= line.n || *n < 1,
                // The start of a line range must be at least 1, and the end must be at least 2.
                // The end of the line range must not be a current or future line.
                LineNumber::Many(r) => {
                    *r.start() < 1 || *r.end() < 2 || *r.end() >= line.n
                }
            })
        {
            return Err(CheckError::BadLine)
        }

        // Ensure all line ranges are citing a valid, complete subproof.
        if line
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
                if (sd < 1 || ed < 1) || (sd != ed) || (sd >= line.n) {
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

        // Accessibility indices for the line being validated.
        let mut sentence_access = vec![false; p.len()];
        let mut subproof_access = vec![false; p.len()];

        // Precompute accessibility relative to all previous lines in the proof.
        // (Present and future lines are by definition inaccessible.)
        //
        // The ceiling value is initialized to the depth of the current line.
        let mut ceil = line.d;

        // Single sentence accessibility.
        // Step backwards through the proof from the current line.
        for n in (1..line.n).rev() {
            let d = p.line(n).map(|l| l.d).unwrap();

            #[allow(clippy::comparison_chain)]
            // If the line's depth is equal to the ceiling value, it is reachable.
            if d == ceil {
                sentence_access[n as usize - 1] = true;
            }
            // If the line is shallower than the ceiling value, it is reachable,
            // but the ceiling is lowered to match.
            else if d < ceil {
                sentence_access[n as usize - 1] = true;
                ceil -= 1;
            }
        }

        let mut ceil = line.d;

        // Subproof accessibility.
        // Similar to above algorithm
        for n in (1..line.n).rev() {
            let l = p.line(n).unwrap();

            // If the line is a premise one level deeper than the current ceiling,
            // then the subproof is reachable.
            if l.d == (ceil + 1) && l.is_premise() {
                subproof_access[n as usize - 1] = true;
            }
            // If the line is shallower than the ceiling value - i.e. we've left a subproof -
            // then the ceiling is lowered to match.
            else if l.d < ceil {
                ceil -= 1;
            }
        }

        // Ensure that no unavailable lines or subproofs are being cited.
        if line
            .cited_lines()
            .iter()
            .any(|ln| match ln {
                LineNumber::One(n) => {
                    !sentence_access[*n as usize - 1]
                }
                LineNumber::Many(r) => {
                    !subproof_access[*r.start() as usize - 1]
                }
            })
        {
            return Err(CheckError::Unavailable)
        }

        if self.strict_only() && !p.strict_zones[line.n as usize - 1] {
            return Err(CheckError::StrictOutside)
        }
        
        if !self.strict_only() && !line.is_premise() && p.strict_zones[line.n as usize - 1] {
            return Err(CheckError::RelaxedInside)
        }

        self.is_right(p, line)?;

        Ok(())
    }
}

// rustc doesn't seem to count uses in default trait method impls?
#[allow(dead_code)]
#[derive(Debug, Error, PartialEq, Eq, Clone)]
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
    BadLine,
    #[error("cited a line range that does not correspond to a subproof")]
    BadRange,
    #[error("cited an unavailable line or subproof")]
    Unavailable,
    #[error("used a strict-subproof-only rule outside of a strict subproof")]
    StrictOutside,
    #[error("used a disallowed rule inside of a strict subproof")]
    RelaxedInside,
}

/* enum Citations<'a> {
    One(&'a Sentence),
    Many(&'a Sentence, &'a Sentence)
} */

fn check_strict_nesting(p: &Proof, s: u16, e: u16) -> Result<(), CheckError> {
    let mut depth = 0_u16;
    let mut nest  = 0_u16;

    for n in s..e {
        let line = p.line(n).unwrap();

        if line.s.is_nec_signal() {
            nest += 1;
        } else if line.d < depth {
            nest = nest.saturating_sub(1);
        }

        depth = line.d;
    }

    if nest > 1 {
        return Err(CheckError::BadUsage)
    }

    Ok(())
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
        let source = l.cited_sentence(p, 0);

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
        let s_a = l.cited_sentence(p, 0);
        let s_b = l.cited_sentence(p, 1);

        let Sentence::Con(lhs, rhs) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if (lhs == s_a || lhs == s_b) && (rhs == s_a || rhs == s_b) {
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
        let source = l.cited_sentence(p, 0);

        let Sentence::Con(lhs, rhs) = source else {
            return Err(CheckError::BadUsage)
        };

        match (lhs == l.s, rhs == l.s) {
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
        let source = l.cited_sentence(p, 0);

        let Sentence::Dis(lhs, rhs) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if (lhs == source) || (rhs == source) {
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
        let source = l.cited_sentence(p, 0);

        let Sentence::Dis(lhs, rhs) = source else {
            return Err(CheckError::BadUsage)
        };

        let (p_1, c_1) = l.cited_subproof(p, 1);
        let (p_2, c_2) = l.cited_subproof(p, 2);

        if (*c_1 != l.s) || (*c_2 != l.s) {
            return Err(CheckError::BadUsage)
        }

        if (p_1 == lhs && p_2 == rhs) || (p_1 == rhs && p_2 == lhs) {
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
        let (p, c) = l.cited_subproof(p, 0);

        let Sentence::Imp(lhs, rhs) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if lhs == p && rhs == c {
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
        let s_1 = l.cited_sentence(p, 0);
        let s_2 = l.cited_sentence(p, 1);
        
        if let Sentence::Imp(lhs, rhs) = s_1 {
            if lhs == s_2 && rhs == l.s {
                return Ok(())
            }
        }

        if let Sentence::Imp(lhs, rhs) = s_2 {
            if lhs == s_1 && rhs == l.s {
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
        let (p_1, c_1) = l.cited_subproof(p, 0);
        let (p_2, c_2) = l.cited_subproof(p, 1);

        let Sentence::Bic(lhs, rhs) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if (lhs == p_1 && rhs == p_2) && (lhs == c_2 && rhs == c_1) {
            return Ok(())
        }

        if (lhs == p_2 && rhs == p_1) && (lhs == c_1 && rhs == c_2) {
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
        let s_1 = l.cited_sentence(p, 0);
        let s_2 = l.cited_sentence(p, 1);

        let Sentence::Bic(lhs, rhs) = s_1 else {
            return Err(CheckError::BadUsage)
        };

        if (lhs == s_2 && rhs == l.s) || (rhs == s_2 && lhs == l.s) {
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
        let (p, c) = l.cited_subproof(p, 0);

        if !c.is_bot_signal() {
            return Err(CheckError::BadUsage)
        };

        if let Sentence::Neg(s) = &l.s {
            if s == p {
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
        let s_1 = l.cited_sentence(p, 0);
        let s_2 = l.cited_sentence(p, 1);

        if !l.s.is_bot_signal() {
            return Err(CheckError::BadUsage)
        };

        if let Sentence::Neg(s_1) = s_1 {
            if s_1 == s_2 {
                return Ok(())
            } else {
                return Err(CheckError::BadUsage)
            }
        }

        if let Sentence::Neg(s_2) = s_2 {
            if s_2 == s_1 {
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
        let source = l.cited_sentence(p, 0);

        if !source.is_bot_signal() {
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
        let (p, c) = l.cited_subproof(p, 0);

        let Sentence::Neg(p) = p else {
            return Err(CheckError::BadUsage)
        };

        if !c.is_bot_signal() {
            return Err(CheckError::BadUsage)
        };

        if p != l.s {
            return Err(CheckError::BadUsage)
        }

        Ok(())
    }
}

struct DisjunctiveSyllogism;

impl Rule for DisjunctiveSyllogism {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One, LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let s_1 = l.cited_sentence(p, 0);
        let s_2 = l.cited_sentence(p, 1);

        if let Sentence::Dis(lhs, rhs) = s_1 {
            let Sentence::Neg(s_2) = s_2 else {
                return Err(CheckError::BadUsage)
            };

            if (s_2 == lhs && l.s == rhs) || (s_2 == rhs && l.s == lhs) {
                return Ok(())
            }
        }
        
        if let Sentence::Dis(lhs, rhs) = s_2 {
            let Sentence::Neg(s_1) = s_1 else {
                return Err(CheckError::BadUsage)
            };

            if (s_1 == lhs && l.s == rhs) || (s_1 == rhs && l.s == lhs) {
                return Ok(())
            }
        }

        Err(CheckError::BadUsage)
    }
}

struct ModusTollens;

impl Rule for ModusTollens {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One, LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let s_1 = l.cited_sentence(p, 0);
        let s_2 = l.cited_sentence(p, 1);

        let Sentence::Neg(s) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if let Sentence::Imp(lhs, rhs) = s_1 {
            let Sentence::Neg(s_2) = s_2 else {
                return Err(CheckError::BadUsage);
            };

            if s == lhs && s_2 == rhs {
                return Ok(())
            }
        }

        if let Sentence::Imp(lhs, rhs) = s_2 {
            let Sentence::Neg(s_1) = s_1 else {
                return Err(CheckError::BadUsage);
            };

            if s == lhs && s_1 == rhs {
                return Ok(())
            }
        }
        
        Err(CheckError::BadUsage)
    }
}

struct Dne;

impl Rule for Dne {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let s = l.cited_sentence(p, 0);
        
        let Sentence::Neg(s) = s else {
            return Err(CheckError::BadUsage)
        };

        let Sentence::Neg(s) = &**s else {
            return Err(CheckError::BadUsage)
        };

        if s == l.s {
            return Ok(())
        }

        Err(CheckError::BadUsage)
    }
}

struct Lem;

impl Rule for Lem {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::Many, LineNumberType::Many]
    }
    
    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let (p_1, c_1) = l.cited_subproof(p, 0);
        let (p_2, c_2) = l.cited_subproof(p, 1);

        if c_1 != c_2 {
            return Err(CheckError::BadUsage)
        }

        if (p_1.negated() != *p_2) && (p_2.negated() != *p_1) {
            return Err(CheckError::BadUsage)
        }

        if &l.s != c_1 {
            return Err(CheckError::BadUsage)
        }

        Ok(())
    }
}

struct DeMorgan;

impl Rule for DeMorgan {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        // this is... something
        match l.cited_sentence(p, 0) {
            Sentence::Neg(inner) => {
                match &**inner {
                    Sentence::Con(lhs, rhs) => {
                        if l.s == Sentence::Dis( lhs.negated().box_up(), rhs.negated().box_up() ) {
                            return Ok(())
                        }
                    },
                    Sentence::Dis(lhs, rhs) => {
                        if l.s == Sentence::Con( lhs.negated().box_up(), rhs.negated().box_up() ) {
                            return Ok(())
                        }
                    },
                    _ => ()
                }
            },
            Sentence::Con(lhs, rhs) => {
                if let ( Sentence::Neg(lhs), Sentence::Neg(rhs) ) = (&**lhs, &**rhs) {
                    if l.s == Sentence::Dis( lhs.clone(), rhs.clone() ).negated() {
                        return Ok(())
                    }
                }
            },
            Sentence::Dis(lhs, rhs) => {
                if let ( Sentence::Neg(lhs), Sentence::Neg(rhs) ) = (&**lhs, &**rhs) {
                    if l.s == Sentence::Con( lhs.clone(), rhs.clone() ).negated() {
                        return Ok(())
                    }
                }
            },
            _ => ()
        }

        Err(CheckError::BadUsage)
    }
}

struct NecessityIntr;

impl Rule for NecessityIntr {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::Many]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let (p, c) = l.cited_subproof(p, 0);

        if !p.is_nec_signal() {
            return Err(CheckError::BadUsage)
        };

        let Sentence::Nec(s) = &l.s else {
            return Err(CheckError::BadUsage)
        };

        if s == c {
            Ok(())
        } else {
            Err(CheckError::BadUsage)
        }
    }
}

struct NecessityElim;

impl Rule for NecessityElim {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn strict_only(&self) -> bool {
        true
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let n = l.cited_lines()[0].as_one();
        let s = l.cited_sentence(p, 0);

        let Sentence::Nec(s) = s else {
            return Err(CheckError::BadUsage)
        };

        check_strict_nesting(p, n, l.n)?;

        if s == l.s {
            return Ok(())
        }

        Err(CheckError::BadUsage)
    }
}

struct PossibilityDef;

impl Rule for PossibilityDef {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {        
        match l.cited_sentence(p, 0) {
            Sentence::Pos(inner) => {
                let Sentence::Neg(s) = &l.s else {
                    return Err(CheckError::BadUsage)
                };

                let Sentence::Nec(s) = &**s else {
                    return Err(CheckError::BadUsage)
                };

                let Sentence::Neg(s) = &**s else {
                    return Err(CheckError::BadUsage)
                };

                if inner == s {
                    return Ok(())
                }
            },
            Sentence::Neg(inner) => {
                let Sentence::Nec(inner) = &**inner else {
                    return Err(CheckError::BadUsage)
                };

                let Sentence::Neg(inner) = &**inner else {
                    return Err(CheckError::BadUsage)
                };

                let Sentence::Pos(s) = &l.s else {
                    return Err(CheckError::BadUsage)
                };

                if inner == s {
                    return Ok(())
                }
            }
            _ => ()
        }

        Err(CheckError::BadUsage)
    }
}

struct ModalConversion;

impl Rule for ModalConversion {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        // love too pattern match
        match l.cited_sentence(p, 0) {
            Sentence::Neg(inner) => {
                match &**inner {
                    Sentence::Nec(inner) => {
                        let Sentence::Pos(s) = &l.s else {
                            return Err(CheckError::BadUsage)
                        };

                        let Sentence::Neg(s) = &**s else {
                            return Err(CheckError::BadUsage)
                        };

                        if inner == s {
                            return Ok(())
                        }
                    },
                    Sentence::Pos(inner) => {
                        let Sentence::Nec(s) = &l.s else {
                            return Err(CheckError::BadUsage)
                        };

                        let Sentence::Neg(s) = &**s else {
                            return Err(CheckError::BadUsage)
                        };

                        if inner == s {
                            return Ok(())
                        }
                    },
                    _ => ()
                }
            },
            Sentence::Pos(inner) => {
                let Sentence::Neg(inner) = &**inner else {
                    return Err(CheckError::BadUsage)
                };

                let Sentence::Neg(s) = &l.s else {
                    return Err(CheckError::BadUsage)
                };

                let Sentence::Nec(s) = &**s else {
                    return Err(CheckError::BadUsage)
                };

                if inner == s {
                    return Ok(())
                }
            },
            Sentence::Nec(inner) => {
                let Sentence::Neg(inner) = &**inner else {
                    return Err(CheckError::BadUsage)
                };

                let Sentence::Neg(s) = &l.s else {
                    return Err(CheckError::BadUsage)
                };

                let Sentence::Pos(s) = &**s else {
                    return Err(CheckError::BadUsage)
                };
                
                if inner == s {
                    return Ok(())
                }
            }
            _ => ()
        }
        
        Err(CheckError::BadUsage)
    }
}

struct RT;

impl Rule for RT {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let s = l.cited_sentence(p, 0);

        let Sentence::Nec(s) = s else {
            return Err(CheckError::BadUsage)
        };

        if s == l.s {
            return Ok(())
        }
        
        Err(CheckError::BadUsage)
    }
}

struct R4;

impl Rule for R4 {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn strict_only(&self) -> bool {
        true
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let n = l.cited_lines()[0].as_one();
        let s = l.cited_sentence(p, 0);

        check_strict_nesting(p, n, l.n)?;

        if s == &l.s {
            return Ok(())
        }

        Err(CheckError::BadUsage)
    }
}

struct R5;

impl Rule for R5 {
    fn line_ord(&self) -> &[LineNumberType] {
        &[LineNumberType::One]
    }

    fn strict_only(&self) -> bool {
        true
    }

    fn is_right(&self, p: &Proof, l: &Line) -> Result<(), CheckError> {
        let n = l.cited_lines()[0].as_one();
        let s = l.cited_sentence(p, 0);
        
        let Sentence::Neg(s_inner) = s else {
            return Err(CheckError::BadUsage)
        };

        let Sentence::Nec(_) = &**s_inner else {
            dbg!();
            return Err(CheckError::BadUsage)
        };

        check_strict_nesting(p, n, l.n)?;

        if s == &l.s {
            return Ok(())
        }

        Err(CheckError::BadUsage)
    }
}