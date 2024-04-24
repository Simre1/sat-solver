use std::collections::BTreeMap;

use dimacs::Var;

pub enum SATResult {
    SAT { model: Model },
    UNSAT,
}

pub struct Model {
    pub assignments: BTreeMap<Var, bool>,
}
