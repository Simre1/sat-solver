use crate::algorithm::interface::SATResult::*;
use crate::algorithm::interface::Assignment::*;
use dimacs::{Clause, Lit, Sign};
use std::collections::{BTreeSet, LinkedList};

use super::interface::*;

pub struct ClauseWatcher {
    watches: [Lit;2],
    clause: Clause,
}

pub struct Clauses {
    clauses: Vec<ClauseWatcher>,
    watches_for_var: Vec<BTreeSet<usize>>,
}


impl Clauses {

    fn from_dimacs(num_vars: usize, dimacs: Box<[Clause]>) -> Clauses {
        let mut clauses = Vec::with_capacity(dimacs.len());
        let mut watches_for_var = vec![BTreeSet::new(); num_vars];

        for (index, clause) in (*dimacs).into_iter().enumerate() {
            // let watch1 = clause
            let watch1 = clause.lits()[0];
            let watch2 = if clause.lits().len() > 1 {
                clause.lits()[1]
            } else {
                clause.lits()[0]
            };

            clauses.push(ClauseWatcher {
                watches: [watch1,watch2],
                clause: clause.clone(),
            });

            watches_for_var[lit_to_index(watch1)].insert(index);
            watches_for_var[lit_to_index(watch2)].insert(index);
        }

        Clauses {
            clauses,
            watches_for_var,
        }
    }
}

pub struct Assignments {
    values: Vec<Assignment>,
}

impl Assignments {
    fn all_unassigned(num_vars: usize) -> Assignments {
        Assignments {
            values: vec![Unassigned; num_vars],
        }
    }

    fn next_unassigned_lit(&self)-> Option<Lit> {
        match self.values.iter().position(|&x| x == Unassigned){
            Some(idx) => Some(Lit::from_i64((idx + 1) as i64)),
            None=>None
        }
    }

    fn unassign(&mut self, lits: &Vec<Lit>){
        for lit in lits {
            self.values[lit_to_index(*lit)] = Unassigned;
        }
    }
}

fn lit_to_index(lit: Lit) -> usize {
    lit.var().to_u64() as usize - 1
}

fn negate(lit: Lit) -> Lit {
    Lit::from_i64(-(lit.var().0 as i64))
}

fn assignment_from_sign(sign: Sign) -> Assignment {
    match sign {
        Sign::Pos => Top,
        Sign::Neg => Bot,
    }
}


fn truth_value_of_literal(assignments: &Assignments, lit: Lit) ->Assignment{
    let assignment = assignments.values[lit_to_index(lit)];
    match (lit.sign(), assignment) {
        (_, Unassigned) => Unassigned,
        (Sign::Neg, Bot) => Top,
        (Sign::Pos, Top) => Top,
        (Sign::Pos, Bot) => Bot,
        (Sign::Neg, Top) => Bot,
    }
}

fn find_next_watched_literal(
    assignments: &Assignments,
    clause: &Clause,
    other_watched: Lit,
) -> Option<Lit> {

    for lit in clause.lits() {
        if assignments.values[lit_to_index(*lit)] == Unassigned
            && *lit != other_watched
        {
            return Some(*lit);
        }
    }
    return None;
}


fn bcp(clauses: &mut Clauses, assignments: &mut Assignments, lit: Lit) -> Option<Vec<Lit>> {
    let mut assigned_lits: Vec<Lit> = vec![lit];

    let mut iterations = 0;
    while iterations < assigned_lits.len() {
        let current_lit = assigned_lits[iterations];
        let i = lit_to_index(current_lit);
        assignments.values[i] = assignment_from_sign(current_lit.sign());
        let mut mapping_changes = LinkedList::new();

        for clause_index in clauses.watches_for_var[i].iter() {
            let clause_watcher = &mut clauses.clauses[*clause_index];

            let watched1 = truth_value_of_literal(assignments, clause_watcher.watches[0]);
            let watched2 = truth_value_of_literal(assignments, clause_watcher.watches[1]);

            match (watched1,watched2) {
                (Top,_)| (_,Top)=>continue,
                (Bot,Bot) =>{assignments.unassign(&assigned_lits); return None;}
                (Bot,_)|(_,Bot) => {
                    let bot_watch_idx = if watched1 == Bot {0} else {1};
                    let other_watch_idx = 1-bot_watch_idx;

                    let next = find_next_watched_literal(
                        assignments,
                        &clause_watcher.clause,
                        clause_watcher.watches[other_watch_idx],
                    );
                    match next {
                        Some(lit) => {
                            clause_watcher.watches[bot_watch_idx] = lit;
                            mapping_changes.push_front((i, lit_to_index(lit), *clause_index));
                        }
                        None => {
                            assigned_lits.push(clause_watcher.watches[other_watch_idx]);
                        }
                    }
                },
                (_,_) => panic!("Should not happen")
            }
        }

        for (from, to, clause_index) in mapping_changes {
            clauses.watches_for_var[from].remove(&clause_index);
            clauses.watches_for_var[to].insert(clause_index);
        }
        iterations+=1;
    }
    return Some(assigned_lits);
}

pub fn dpll_algorithm(num_vars: usize, clauses: Box<[Clause]>) -> SATResult {
    let mut clauses = Clauses::from_dimacs(num_vars, clauses);
    let mut assignments = Assignments::all_unassigned(num_vars);

    dpll_recursive(&mut clauses, &mut assignments)
}

fn dpll_recursive(clauses: &mut Clauses, assignments: &mut Assignments) -> SATResult{
    match assignments.next_unassigned_lit() {
        None =>{
            return SAT {
                model: Model {
                    assignments: assignments
                        .values
                        .iter()
                        .map(|x| x.to_bool())
                        .collect(),
                },
            };
        }
        Some(next_lit)=>{
            for lit in [next_lit, negate(next_lit)]{
                match bcp(clauses, assignments, lit) {
                    Some(assigned) => {
                        match dpll_recursive(clauses, assignments){
                            SAT{model} => return SAT{model},
                            UNSAT => assignments.unassign(&assigned)
                        }
                    }
                    None =>()
                }
            }
            return UNSAT;
        }
    }
}
