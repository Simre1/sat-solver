use dimacs::Clause;
use dimacs::Sign;
use dimacs::Lit;
use super::interface::Model;
use super::interface::SATResult;

#[derive(Clone, PartialEq, Debug)]
pub enum Assignment{
    Top,
    Bot,
    Unassigned
}

impl Assignment {
    fn to_bool(&self) -> bool {
        match self {
            Assignment::Top => true,
            Assignment::Bot => false,
            Assignment::Unassigned => panic!("cannot convert Unassigned to bool")
        }
    }
}

pub fn simple_algorithm(num_vars: u64, clauses: Box<[Clause]>) -> SATResult {
    let mut assignment = vec![Assignment::Unassigned;num_vars as usize];
    let sat = simple_algorithm_recursion(num_vars as usize, &clauses, &mut assignment, 0);

    if sat {
        let bool_assignment = assignment.iter().map(|a| a.to_bool()).collect();
        return SATResult::SAT { model: Model{assignments : bool_assignment} };
    }
    else{
        return SATResult::UNSAT;
    }
}

fn simple_algorithm_recursion(num_vars: usize, clauses: &Box<[Clause]>, assignment:  &mut Vec<Assignment>, cur_var: usize) -> bool{
    if cur_var >= num_vars {  return true;}

    for value in [Assignment::Top, Assignment::Bot].into_iter() {
        assignment[cur_var] = value;
        if !contains_false_clause(clauses, assignment) {
            let sat = simple_algorithm_recursion(num_vars, clauses, assignment, cur_var+1);
            if sat {return true};
        }
    }

    assignment[cur_var] = Assignment::Unassigned;
    false
}

fn contains_false_clause(clauses: &Box<[Clause]>, assignment: &Vec<Assignment>) -> bool{
    clauses.iter().any(|c| false_clause(c, assignment))
}

fn false_clause(clause: &Clause, assignment:  &Vec<Assignment>)->bool{
    clause.lits().iter().all(|l| false_assignment(l, assignment))
}

fn false_assignment(lit: &Lit, assignment: &Vec<Assignment>)->bool{
    let var_idx = lit.var().0 as usize;
    (lit.sign() == Sign::Neg && assignment[var_idx] == Assignment::Top) || 
    (lit.sign() == Sign::Pos && assignment[var_idx] == Assignment::Bot)
}