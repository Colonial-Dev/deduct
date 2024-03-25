use once_cell::sync::Lazy;
use regex::Regex;

mod citation;
mod sentence;

pub mod consts {
    pub const NEG: &str = "¬";
    pub const CON: &str = "∧";
    pub const DIS: &str = "∨";
    pub const BIC: &str = "↔";
    pub const IMP: &str = "→";
    pub const BOT: &str = "⊥";
    pub const NEC: &str = "□";
    pub const POS: &str = "◇";
}

pub use sentence::*;

/// Normalize operator shorthands.
fn normalize_ops(i: &str) -> String {
    use std::ops::Deref;
    use consts::*;
    
    static BIC_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:≡|<\->|<>)"#).unwrap(), BIC) );
    static IMP_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:⇒|⊃|->|>)"#).unwrap(), IMP) );
    static CON_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:\^|&|\.|·|\*)"#).unwrap(), CON) );
    static DIS_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"v"#).unwrap(), DIS) );
    static NEG_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:~|∼|-|−)"#).unwrap(), NEG) );
    static BOT_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"(?:XX|#)"#).unwrap(), BOT) );
    static NEC_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"\[\]"#).unwrap(), NEC) );
    static POS_REGEX: Lazy<(Regex, &'static str)> = Lazy::new(|| (Regex::new(r#"<>"#).unwrap(), POS) );
        
    let pairs = [
        BIC_REGEX.deref(),
        IMP_REGEX.deref(),
        CON_REGEX.deref(),
        DIS_REGEX.deref(),
        NEG_REGEX.deref(),
        BOT_REGEX.deref(),
        NEC_REGEX.deref(),
        POS_REGEX.deref(),
    ];

    let mut out = i.to_owned();

    for (regex, norm) in pairs {
        out = regex.replace_all(&out, *norm).to_string();
    }

    out
}