use dimacs::Clause;

use super::interface::SATResult;

pub fn simple_algorithm(num_vars: u64, clauses: Box<[Clause]>) -> SATResult {
    // TODO: Fill in algorithm here
    SATResult::UNSAT
}
