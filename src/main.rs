mod parse;

pub use crate::parse::*;

fn main() {
    println!("{:#?}", Sentence::parse("◇(A -> B) ^ (C <-> D)"));
}



struct Citation;
struct Line {
    s: Sentence,
    c: Citation,
    l: u16,
    d: u16
}