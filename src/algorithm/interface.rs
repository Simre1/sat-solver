use SATResult::*;
pub enum SATResult {
    SAT { model: Model },
    UNSAT,
}
impl PartialEq for SATResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SAT { model: _ }, SAT { model: _ }) => true,
            (UNSAT, UNSAT) => true,
            _ => false,
        }
    }
}

pub struct Model {
    pub assignments: Vec<bool>,
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum Assignment {
    Top,
    Bot,
    Unassigned,
}

impl Assignment {
    pub fn to_bool(&self) -> bool {
        match self {
            Assignment::Top => true,
            Assignment::Bot => false,
            Assignment::Unassigned => panic!("cannot convert Unassigned to bool"),
        }
    }
}
