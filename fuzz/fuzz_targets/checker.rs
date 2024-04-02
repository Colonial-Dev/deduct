#![no_main]

extern crate deduct;

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary;

use deduct::*;

static CHECKER: OnceLock<Checker> = OnceLock::new();

#[derive(Debug, arbitrary::Arbitrary)]
struct ArbProof<'a> {
    inner: Vec<(u16, &'a str, &'a str)>
}

fuzz_target!(|data: ArbProof| {
    let c = CHECKER.get_or_init(|| {
        let mut c = Checker::new();
        c.add_ruleset(TFL_BASIC);
        c.add_ruleset(TFL_DERIVED);
        c.add_ruleset(SYSTEM_K);
        c.add_ruleset(SYSTEM_T);
        c.add_ruleset(SYSTEM_S4);
        c.add_ruleset(SYSTEM_S5);
        c
    });

    if let Ok(p) = Proof::parse(&data.inner) {
        let _ = c.check_proof(&p);
    }
});
