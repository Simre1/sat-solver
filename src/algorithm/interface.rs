use dimacs::{Clause, Lit, Sign};

pub enum SATResult {
    SAT { model: Model },
    UNSAT,
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

pub fn lit_to_index(lit: Lit) -> usize {
    lit.var().to_u64() as usize - 1
}

pub fn index_to_lit(index: usize, assignment: Assignment) -> Lit {
    match assignment {
        Assignment::Top => Lit::from_i64(index as i64 + 1),
        Assignment::Bot => Lit::from_i64(-(index as i64 + 1)),
        Assignment::Unassigned => panic!("Cannot create unassigned lit from index {}", index),
    }
}

pub fn negate(lit: Lit) -> Lit {
    match lit.sign() {
        Sign::Pos => Lit::from_i64(-(lit.var().0 as i64)),
        Sign::Neg => Lit::from_i64(lit.var().0 as i64),
    }
}

pub fn assignment_from_sign(sign: Sign) -> Assignment {
    match sign {
        Sign::Pos => Assignment::Top,
        Sign::Neg => Assignment::Bot,
    }
}
