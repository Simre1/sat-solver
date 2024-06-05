use std::collections::HashSet;
use dimacs::{Clause, Instance, Lit, parse_dimacs, Sign};

use crate::algorithm::interface::Assignment;

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

pub fn preprocess(dimacs: &Box<[Clause]>) -> Box<[Clause]>{
    let mut clauses = Vec::new();
    for clause in dimacs.iter(){
        match preprocess_clause(clause)
        {
            Some(cleaned) => clauses.push(cleaned),
            None => ()
        }
    }
    return clauses.into_boxed_slice();
}

fn preprocess_clause(clause: &Clause) -> Option<Clause>{
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();

    for &lit in clause.lits().iter() {
        if !seen.insert(lit) {
            duplicates.push(lit);
        }
        //Tautology
        if seen.contains(&negate(lit)) {
            return None
        }
    }

    let cleaned_clause = Clause::from_vec(seen.into_iter().collect());
    if cleaned_clause.len() == 0 {return None}
    return Some(cleaned_clause);
}

pub fn check_result(clauses: &Box<[Clause]>, assignment:  &Vec<bool>)->bool{
    clauses.iter().all(|clause| clause_true(clause, assignment) )
}

fn clause_true(clause: &Clause,  assignment:  &Vec<bool>)->bool{
    clause.lits().iter().any(|lit| true_assignment(lit, assignment) )
}

fn true_assignment(lit: &Lit, assignment: &Vec<bool>)->bool{
    let var_idx = (lit.var().0-1) as usize;
    (lit.sign() == Sign::Neg && assignment[var_idx] == false) ||
        (lit.sign() == Sign::Pos && assignment[var_idx] == true)
}

pub fn read_file(file: &str) -> (usize, Box<[Clause]>) {

    // Parse CNF input
    let instance = match parse_dimacs(file){
        Ok(data) => data,
        Err(_) => {
            println!("could not parse file: {:?}", file);
            panic!();
        }
    };
    let (num_vars, clauses) = match instance {
        Instance::Cnf { num_vars, clauses } => (num_vars as usize, clauses),
        Instance::Sat {
            num_vars: _,
            extensions: _,
            formula: _,
        } => panic!("SAT files are not supported, only DIMACS files"),
    };
    (num_vars, clauses)
}