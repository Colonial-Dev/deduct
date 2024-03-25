#![no_main]

extern crate deduct;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = deduct::Citation::parse(s);
    }}
);
