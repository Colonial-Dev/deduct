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
    fn identity(&self) -> &'static str;
    fn validate(&self, p: &Proof, l: u16) -> bool;
}