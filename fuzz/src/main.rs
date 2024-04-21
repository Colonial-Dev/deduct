#[macro_use]
extern crate afl;
extern crate deduct;

use std::sync::OnceLock;

use deduct::*;

static CHECKER: OnceLock<Checker> = OnceLock::new();

fn main() {
    fuzz!(|data: &[u8]| {
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

        let Ok(data) = std::str::from_utf8(&data) else {
            return
        };
    
        let chunks: Vec<_> = data.split(',').collect();
    
        if chunks.chunks_exact(3).remainder().len() != 0 {
            return;
        }
    
        let mut input = Vec::new();
    
        for chunk in chunks.chunks_exact(3) {
            let d = chunk[0].trim();
            let s = chunk[1].trim();
            let c = chunk[2].trim();
    
            let Ok(d) = d.parse::<u16>() else {
                return
            };
    
            input.push(
                (d, s, c)
            )
        }
    
        if let Ok(p) = Proof::parse(input) {
            let _ = c.check_proof(&p);
        }
    });
}
