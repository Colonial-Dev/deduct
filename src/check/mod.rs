pub mod rulesets;
mod rules;

use std::collections::HashMap;

use crate::parse::*;
use crate::check::rules::*;

pub type CheckErrors = Vec<(u16, CheckError)>;
pub type Ruleset<'a> = &'a [(&'static str, &'static dyn Rule)];

pub struct Checker {
    rules: HashMap<&'static str, &'static dyn Rule>
}

impl Checker {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let rules = HashMap::from(
            [("PR", &Premise as &dyn Rule), ("?", &Premise as &dyn Rule)]
        );

        Self { rules }
    }
    
    pub fn add_ruleset(&mut self, ruleset: Ruleset) {
        for (id, rule) in ruleset {
            self.rules.insert(id, *rule);
        }
    }

    pub fn del_ruleset(&mut self, ruleset: Ruleset) {
        for (id, _) in ruleset {
            self.rules.remove(id);
        }
    }

    pub fn check_proof(&self, p: &Proof) -> Result<(), CheckErrors> {
        let mut errors = Vec::new();
        
        for line in &p.lines {
            let Some(rule) = self.rules.get( line.c.r.as_str() ) else {
                errors.push( (line.n, CheckError::NoSuchRule) );
                continue;
            };

            if let Err(e) = rule.validate(p, line) {
                errors.push( (line.n, e) )
            }
        }

        if !errors.is_empty() {
            return Err(errors)
        }

        Ok(())
    }
}

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::rulesets::*;

    macro_rules! proof {
        ([$($rules:ident),+ $(,)?], $($n:literal, $s:literal, $c:literal,)+) => {
            let p = Proof::parse([ $(($n, $s, $c),)+]).expect("Failed to parse test proof");
            let r = [$($rules),+];

            let mut c = Checker::new();

            for ruleset in r {
                c.rules.extend( ruleset.iter().map(|(i, r)| (*i, *r)) );
            }

            c.check_proof(&p).unwrap();
        };
    }

    macro_rules! bad_proof {
        ([$($rules:ident),+ $(,)?], [$($errs:expr),+ $(,)?], $($n:literal, $s:literal, $c:literal,)+) => {
            let p = Proof::parse([ $(($n, $s, $c),)+]).expect("Failed to parse test proof");
            let r = [$($rules),+];
            let e = [$($errs),+];

            let mut c = Checker::new();

            for ruleset in r {
                c.rules.extend( ruleset.iter().map(|(i, r)| (*i, *r)) );
            }

            let errs = c
                .check_proof(&p)
                .unwrap_err()
                .into_iter();

            if errs
                .clone()
                .into_iter()
                .zip( e.clone() )
                .any(|(a, e)| a != e) 
            {
                panic!("Error mismatch!\nExpected {e:?}\nGot:{errs:?}")
            }
        };
    }

    #[test]
    fn reiteration() {
        proof! {
            [TFL_BASIC],
            0, "A", "PR",
            0, "A", "R 1",
        };
    }

    #[test]
    fn conjunction_intr() {
        proof! {
            [TFL_BASIC],
            0, "A", "PR",
            0, "B", "PR",
            0, "A ^ B", "^I 1 2",
            0, "B ^ A", "^I 1 2",
        };
    }

    #[test]
    fn conjunction_elim() {
        proof! {
            [TFL_BASIC],
            0, "A ^ B", "PR",
            0, "A", "^E 1",
            0, "B", "^E 1",
        }
    }

    #[test]
    fn disjunction_intr() {
        proof! {
            [TFL_BASIC],
            0, "A", "PR",
            0, "A v B", "vI 1",
            0, "B v A", "vI 1",
        }
    }

    #[test]
    fn disjunction_elim() {
        proof! {
            [TFL_BASIC],
            0, "A v B", "PR",
            0, "C", "PR",
            1, "A", "PR",
            1, "C", "R 2",
            1, "B", "PR",
            1, "C", "R 2",
            0, "C", "vE 1 3-4 5-6",
            0, "C", "vE 1 5-6 3-4",
        }
    }

    #[test]
    fn conditional_intr() {
        proof! {
            [TFL_BASIC],
            0, "B", "PR",
            1, "A", "PR",
            1, "B", "R 1",
            0, "A -> B", "->I 2-3",
        }
    }

    #[test]
    fn conditional_elim() {
        proof! {
            [TFL_BASIC],
            0, "A -> B", "PR",
            0, "A", "PR",
            0, "B", "->E 1 2",
            0, "B", "->E 2 1",
        }
    }

    #[test]
    fn biconditional_intr() {
        proof! {
            [TFL_BASIC],
            0, "A", "PR",
            0, "B", "PR",
            1, "A", "PR",
            1, "B", "R 2",
            1, "B", "PR",
            1, "A", "R 1",
            0, "A <-> B", "<->I 3-4 5-6",
            0, "B <-> A", "<->I 3-4 5-6",
            0, "A <-> B", "<->I 5-6 3-4",
            0, "B <-> A", "<->I 5-6 3-4",
        }
    }

    #[test]
    fn biconditional_elim() {
        proof! {
            [TFL_BASIC],
            0, "A <-> B", "PR",
            0, "A", "PR",
            0, "B", "PR",
            0, "B", "<->E 1 2",
            0, "A", "<->E 1 3",
        }
    }

    #[test]
    fn negation_intr() {
        proof! {
            [TFL_BASIC],
            0, "~A", "PR",
            1, "A", "PR",
            1, "#", "~E 1 2",
            1, "#", "~E 2 1",
            0, "~A", "~I 2-4",
        }
    }

    #[test]
    fn negation_elim() {
        proof! {
            [TFL_BASIC],
            0, "A", "PR",
            0, "~A", "PR",
            0, "#", "~E 1 2",
            0, "#", "~E 2 1",
        }

        proof! {
            [TFL_BASIC],
            0, "~A", "PR",
            0, "~~A", "PR",
            0, "#", "~E 1 2",
            0, "#", "~E 2 1",
        }

        proof! {
            [TFL_BASIC],
            0, "~~~~~A", "PR",
            0, "~~~~~~A", "PR",
            0, "#", "~E 1 2",
            0, "#", "~E 2 1",
        }
    }

    #[test]
    fn indirect_proof() {
        proof! {
            [TFL_BASIC],
            0, "A", "PR",
            1, "~A", "PR",
            1, "#", "~E 1 2",
            0, "A", "IP 2-3",
        }

        proof! {
            [TFL_BASIC],
            0, "~A", "PR",
            1, "~~A", "PR",
            1, "#", "~E 1 2",
            0, "~A", "IP 2-3",
        }
    }

    #[test]
    fn explosion() {
        proof! {
            [TFL_BASIC],
            0, "A", "PR",
            0, "~A", "PR",
            0, "#", "~E 1 2",
            0, "(B ^ O) ^ (O ^ M)", "X 3",
        }
    }

    #[test]
    fn complex_tfl_basic() {
        // Homework 2-1
        // Prove ~B from ~(B <-> A) and A
        proof! {
            [TFL_BASIC],
            0, "~(B <-> A)", "PR",
            0, "A", "PR",
            1, "B", "PR",
            2, "A", "PR",
            2, "B", "R 3",
            2, "B", "PR",
            2, "A", "R 2",
            1, "B <-> A", "<->I 4-5, 6-7",
            1, "#", "~E 1 8",
            0, "~B", "~I 3-9",
        }

        // Homework 2-2
        // Prove ~B -> ~A from A -> (B v C) and B <-> C
        proof! {
            [TFL_BASIC],
            0, "A -> (B v C)", "PR",
            0, "B <-> C", "PR",
            1, "~B", "PR",
            2, "A", "PR",
            2, "B v C", "->E 1 4",
            3, "B", "PR",
            3, "B", "R 6",
            3, "C", "PR",
            3, "B", "<->E 2 8",
            2, "B", "vE 5 6-7 8-9",
        }

        // Homework 2-3
        // Theorem: prove C -> ([(D ^ A) v B] -> C)
        proof! {
            [TFL_BASIC],
            1, "C", "PR",
            2, "(D ^ A) v B", "PR",
            2, "C", "R 1",
            1, "((D ^ A) v B) -> C", "->I 2-3",
            0, "C -> ([(D ^ A) v B] -> C)", "->I 1-4",
        }

        // Homework 2-4
        // Prove A ^ (B ^ C) from (A ^ B) ^ C
        proof! {
            [TFL_BASIC],
            0, "(A ^ B) ^ C", "PR",
            0, "A ^ B", "^E 1",
            0, "A", "^E 2",
            0, "B", "^E 2",
            0, "C", "^E 1",
            0, "(B ^ C)", "^I 4 5",
            0, "A ^ (B ^ C)", "^I 3 6",
        }

        // Homework 2-5
        // Prove A v (B v C) from (A v B) v C
        proof! {
            [TFL_BASIC],
            0, "(A v B) v C", "PR",
            1, "A v B", "PR",
            2, "A", "PR",
            2, "A v (B v C)", "vI 3",
            2, "B", "PR",
            2, "B v C", "vI 5",
            2, "A v (B v C)", "vI 6",
            1, "A v (B v C)", "vE 2 3-4 5-7",
            1, "C", "PR",
            1, "B v C", "vI 9",
            1, "A v (B v C)", "vI 10",
            0, "A v (B v C)", "vE 1 2-8 9-11",
        }
    }

    #[test]
    fn disjunctive_syllogism() {
        proof! {
            [TFL_BASIC, TFL_DERIVED],
            0, "A v B", "PR",
            0, "~A", "PR",
            0, "~B", "PR",
            0, "A", "DS 1 3",
            0, "B", "DS 1 2",
            0, "A", "DS 3 1",
            0, "B", "DS 2 1",
        }
    }

    #[test]
    fn modus_tollens() {
        proof! {
            [TFL_BASIC, TFL_DERIVED],
            0, "A -> B", "PR",
            0, "~B", "PR",
            0, "~A", "MT 1 2",
            0, "~A", "MT 2 1",
        }

        proof! {
            [TFL_BASIC, TFL_DERIVED],
            0, "A -> ~B", "PR",
            0, "~~B", "PR",
            0, "~A", "MT 1 2",
            0, "~A", "MT 2 1",
        }
    }

    #[test]
    fn dne() {
        proof! {
            [TFL_BASIC, TFL_DERIVED],
            0, "~~A", "PR",
            0, "~~~B", "PR",
            0, "A", "DNE 1",
            0, "~B", "DNE 2",
        }
    }

    #[test]
    fn lem() {
        proof! {
            [TFL_BASIC, TFL_DERIVED],
            0, "B", "PR",
            1, "A", "PR",
            1, "B", "R 1",
            1, "~A", "PR",
            1, "B", "R 1",
            0, "B", "LEM 2-3 4-5",
            0, "B", "LEM 4-5 2-3",
        }
    }

    #[test]
    fn de_morgan() {
        proof! {
            [TFL_BASIC, TFL_DERIVED],
            0, "~(A v B)", "PR",
            0, "~A ^ ~B", "DeM 1",
        }

        proof! {
            [TFL_BASIC, TFL_DERIVED],
            0, "~A ^ ~B", "PR",
            0, "~(A v B)", "DeM 1",
        }

        proof! {
            [TFL_BASIC, TFL_DERIVED],
            0, "~(A ^ B)", "PR",
            0, "~A v ~B", "DeM 1",
        }

        proof! {
            [TFL_BASIC, TFL_DERIVED],
            0, "~A v ~B", "PR",
            0, "~(A ^ B)", "DeM 1",
        }
    }

    #[test]
    fn complex_tfl_derived() {

    }

    #[test]
    fn necessity_intr() {
        proof! {
            [TFL_BASIC, SYSTEM_K],
            0, "[]A", "PR",
            1, "[]", "PR",
            1, "A", "[]E 1",
            0, "[]A", "[]I 2-3",
        }
    }

    #[test]
    fn necessity_elim() {
        proof! {
            [SYSTEM_K],
            0, "[]A", "PR",
            1, "[]", "PR",
            1, "A", "[]E 1",
        }

        bad_proof! {
            [TFL_BASIC, SYSTEM_K],
            [(4, CheckError::BadUsage)],
            0, "[]A", "PR",
            1, "[]", "PR",
            2, "[]", "PR",
            2, "A", "[]E 1",
        }
    }

    #[test]
    fn possibility_def() {
        proof! {
            [SYSTEM_K],
            0, "~[]~A", "PR",
            0, "<>A", "Def<> 1",
        }

        proof! {
            [SYSTEM_K],
            0, "<>A", "PR",
            0, "~[]~A", "Def<> 1",
        }
    }

    #[test]
    fn modal_conversion() {
        proof! {
            [SYSTEM_K],
            0, "~[]A", "PR",
            0, "<>~A", "MC 1",
            0, "~[]A", "MC 2",
        }

        proof! {
            [SYSTEM_K],
            0, "~<>A", "PR",
            0, "[]~A", "MC 1",
            0, "~<>A", "MC 2",
        }
    }

    #[test]
    fn rule_t() {
        proof! {
            [SYSTEM_T],
            0, "[]A", "PR",
            0, "A", "RT 1",
        }

        bad_proof! {
            [SYSTEM_T],
            [(3, CheckError::Unavailable)],
            0, "[]A", "PR",
            1, "[]", "PR",
            1, "[]A", "RT 1",
        }
    }

    #[test]
    fn rule_four() {
        proof! {
            [SYSTEM_S4],
            0, "[]A", "PR",
            1, "[]", "PR",
            1, "[]A", "R4 1",
        }

        bad_proof! {
            [SYSTEM_S4],
            [(4, CheckError::BadUsage)],
            0, "[]A", "PR",
            1, "[]", "PR",
            2, "[]", "PR",
            2, "[]A", "R4 1",
        }
    }

    #[test]
    fn rule_five() {
        proof! {
            [SYSTEM_S5],
            0, "~[]A", "PR",
            1, "[]", "PR",
            1, "~[]A", "R5 1",
        }

        bad_proof! {
            [SYSTEM_S5],
            [(4, CheckError::BadUsage)],
            0, "~[]A", "PR",
            1, "[]", "PR",
            2, "[]", "PR",
            2, "~[]A", "R5 1",
        }
    }

    #[test]
    fn complex_modal() {
        // Homework 5-5
        // Prove [](P v R) from []P
        proof! {
            [TFL_BASIC, SYSTEM_K],
            0, "[]P", "PR",
            1, "[]", "PR",
            1, "P", "[]E 1",
            1, "P v R", "vI 3",
            0, "[](P v R)", "[]I 2-4",
        }

        // Homework 5-6
        // Prove [](P -> R) from [](P -> Q) and [](Q -> P)
        proof! {
            [TFL_BASIC, SYSTEM_K],
            0, "[](P -> Q)", "PR",
            0, "[](Q -> R)", "PR",
            1, "[]", "PR",
            1, "P -> Q", "[]E 1",
            1, "Q -> R", "[]E 2",
            2, "P", "PR",
            2, "Q", "->E 4 6",
            2, "R", "->E 5 7",
            1, "P -> R", "->I 6-8",
            0, "[](P -> R)", "[]I 3-9",
        }

        // Homework 5-8
        // Prove ~<>P from [](P -> Q) and []~Q
        proof! {
            [TFL_BASIC, SYSTEM_K],
            0, "[](P -> Q)", "PR",
            0, "[]~Q", "PR",
            1, "[]", "PR",
            1, "P -> Q", "[]E 1",
            1, "~Q", "[]E 2",
            2, "P", "PR",
            2, "Q", "->E 4 6",
            2, "#", "~E 5 7",
            1, "~P", "~I 6-8",
            0, "[]~P", "[]I 3-9",
            0, "~<>P", "MC 10",
        }
    }
}