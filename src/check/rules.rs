use thiserror::Error;

use crate::parse::*;

pub struct RuleIndex;

// BASIC
// - Reiteration
// - Conjunction I/E
// - Disjunction I/E
// - Conditional I/E
// - Biconditional I/E
// - Negation I
// - Negation E (contradiction)
// - Explosion
// - Indirect proof
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

        // Ensure expected line number types match the actual
        if self
            .line_ord()
            .iter()
            .zip( l.cited_lines() )
            .any(|(e, a)| e != a) 
        {
            return Err(CheckError::BadLineType)
        }

        // Ensure we are not citing ourselves or the future
        if l
            .cited_lines()
            .iter()
            .map(|l| match l {
                LineNumber::One(n) => n,
                LineNumber::Many(r) => r.start()
            })
            .any(|n| *n >= l.l)
        {
            return Err(CheckError::CitingFuture)
        }

        // TODO precheck for citing lines in closed subproofs
        // TODO precheck for line ranges that do not cite a subproof
        // TODO precheck for citing non-existent lines

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
    #[error("cited a line that does not exist")]
    NoSuchLine,
    #[error("cited a rule that was used incorrectly")]
    BadUsage,
    #[error("cited the current line or one that occurs in the future")]
    CitingFuture,
    #[error("cited a line in a closed subproof")]
    CitingClosed,
}

fn cited_sentence<'a>(p: &'a Proof, l: &Line, n: usize) -> Result<&'a Sentence, CheckError> {
    let Some(l) = p.line( l.cited_lines()[n].as_one() ) else {
        return Err(CheckError::NoSuchLine)
    };

    Ok(&l.s)
}

fn cited_subproof<'a>(p: &'a Proof, l: &Line, n: usize) -> Result<(&'a Sentence, &'a Sentence), CheckError> {
    todo!()
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

        let (pa, ca) = cited_subproof(p, l, 1)?;
        let (pb, cb) = cited_subproof(p, l, 2)?;

        if (*ca != l.s) || (*cb != l.s) {
            return Err(CheckError::BadUsage)
        }

        if !((*pa == **lhs || *pa == **rhs) && (*pb == **lhs || *pb == **rhs)) {

        }

        todo!()
    }
}