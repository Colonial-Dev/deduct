mod rules;

use std::collections::HashMap;

use crate::parse::*;

use self::rules::*;

pub struct Checker {
    rules: HashMap<&'static str, &'static dyn Rule>
}

impl Checker {
    pub fn new() -> Self {
        let rules = HashMap::from(
            [("PR", &Premise as &dyn Rule)]
        );

        Self { rules }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}