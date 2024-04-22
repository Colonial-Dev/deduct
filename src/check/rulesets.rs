use super::rules::*;

pub const ALL_RULESETS: &[&[(&str, &dyn Rule)]] = &[
    TFL_BASIC,
    TFL_DERIVED,
    SYSTEM_K,
    SYSTEM_T,
    SYSTEM_S4,
    SYSTEM_S5
];

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