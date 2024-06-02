use dimacs::{Clause, Lit, Sign};

pub enum SATResult {
    SAT { model: Model },
    UNSAT,
}

pub struct Model {
    pub assignments: Vec<bool>,
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum Assignment{
    Top,
    Bot,
    Unassigned
}

impl Assignment {
    pub fn to_bool(&self) -> bool {
        match self {
            Assignment::Top => true,
            Assignment::Bot => false,
            Assignment::Unassigned => panic!("cannot convert Unassigned to bool")
        }
    }
}

pub fn lit_to_index(lit: Lit) -> usize {
    lit.var().to_u64() as usize - 1
}

pub fn negate(lit: Lit) -> Lit {
    Lit::from_i64(-(lit.var().0 as i64))
}

pub fn assignment_from_sign(sign: Sign) -> Assignment {
    match sign {
        Sign::Pos => Assignment::Top,
        Sign::Neg => Assignment::Bot,
    }
}
