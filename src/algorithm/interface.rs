pub enum SATResult {
    SAT { model: Model },
    UNSAT,
}

pub struct Model {
    pub assignments: Vec<bool>,
}
