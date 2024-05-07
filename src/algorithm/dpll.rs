use crate::algorithm::interface::SATResult::*;
use dimacs::{Clause, Lit, Sign, Var};
use std::collections::{BTreeMap, BTreeSet, LinkedList};
use std::io::read_to_string;

use super::interface::*;

pub struct ClauseWatcher {
    watch1: Lit,
    watch2: Lit,
    clause: Clause,
}

pub struct Clauses {
    clauses: Vec<ClauseWatcher>,
    watches_for_var: Vec<BTreeSet<usize>>,
}

fn lit_to_index(lit: Lit) -> usize {
    lit.var().to_u64() as usize - 1
}

fn index_to_positive_lit(index: usize) -> Lit {
    Lit::from_i64((index + 1) as i64)
}

impl Clauses {
    fn get_mapping(&mut self, var_index: usize) -> &mut BTreeSet<usize> {
        &mut self.watches_for_var[var_index as usize - 1]
    }

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
                watch1,
                watch2,
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

#[derive(Debug)]
pub struct Assignments {
    assignments: Vec<Assignment>,
}

impl Assignments {
    fn all_unassigned(num_vars: usize) -> Assignments {
        Assignments {
            assignments: vec![Assignment::Unassigned; num_vars],
        }
    }
}

fn assignment_from_sign(sign: Sign) -> Assignment {
    match sign {
        Sign::Pos => Assignment::Top,
        Sign::Neg => Assignment::Bot,
    }
}

fn apply_sign(sign: Sign, assignment: Assignment) -> Assignment {
    match (sign, assignment) {
        (Sign::Pos, assignment) => assignment,
        (Sign::Neg, Assignment::Top) => Assignment::Bot,
        (Sign::Neg, Assignment::Bot) => Assignment::Top,
        (Sign::Neg, Assignment::Unassigned) => Assignment::Unassigned,
    }
}

fn find_next_watched_literal(
    assignments: &Assignments,
    clause: &Clause,
    other_watched: Lit,
) -> Option<Lit> {
    for lit in clause.lits() {
        if assignments.assignments[lit_to_index(*lit)] == Assignment::Unassigned
            || *lit == other_watched
        {
            return Some(*lit);
        }
    }
    return None;
}

// fn B
fn bcp(clauses: &mut Clauses, assignments: &mut Assignments, lit: Lit) -> Option<Vec<Lit>> {
    let mut assignment_stack: Vec<Lit> = Vec::new();

    assignment_stack.push(lit);

    let mut iterations = 0;
    while (iterations < assignment_stack.len()) {
        let current_lit = assignment_stack[iterations];
        let i = lit_to_index(current_lit);
        assignments.assignments[i] = assignment_from_sign(current_lit.sign());
        let mut mapping_changes = LinkedList::new();

        for clauseIndex in clauses.watches_for_var[i].iter() {
            let clauseWatcher = &mut clauses.clauses[*clauseIndex];

            let assignment1 = assignments.assignments[lit_to_index(clauseWatcher.watch1)];
            let assignment2 = assignments.assignments[lit_to_index(clauseWatcher.watch2)];
            let sign1 = clauseWatcher.watch1.sign();
            let sign2 = clauseWatcher.watch2.sign();

            let watched1 = apply_sign(sign1, assignment1);
            let watched2 = apply_sign(sign2, assignment2);

            if watched1 == Assignment::Top || watched2 == Assignment::Top {
                continue;
            }

            if watched1 == Assignment::Bot && watched2 == Assignment::Bot {
                for wrongly_assigned in assignment_stack {
                    assignments.assignments[lit_to_index(wrongly_assigned)] =
                        Assignment::Unassigned;
                }
                return None;
            }

            if watched1 == Assignment::Bot {
                let next = find_next_watched_literal(
                    assignments,
                    &clauseWatcher.clause,
                    clauseWatcher.watch2,
                );
                match next {
                    Some(lit) => {
                        clauseWatcher.watch1 = lit;
                        mapping_changes.push_front((i, lit_to_index(lit), *clauseIndex));
                    }
                    None => {
                        assignment_stack.push(clauseWatcher.watch2);
                    }
                }
            }

            if watched2 == Assignment::Bot {
                let next = find_next_watched_literal(
                    assignments,
                    &clauseWatcher.clause,
                    clauseWatcher.watch1,
                );
                match next {
                    Some(lit) => {
                        clauseWatcher.watch2 = lit;
                        mapping_changes.push_front((i, lit_to_index(lit), *clauseIndex));
                    }
                    None => {
                        assignment_stack.push(clauseWatcher.watch1);
                    }
                }
            }

            if watched1 == Assignment::Unassigned && watched2 == Assignment::Unassigned {
                panic!("Should not happen")
            }
        }

        for (from, to, clauseIndex) in mapping_changes {
            clauses.watches_for_var[from].remove(&clauseIndex);
            clauses.watches_for_var[to].insert(clauseIndex);
        }
        iterations += 1;
    }
    return Some(assignment_stack);
}

pub fn dpll_algorithm(num_vars: usize, clauses: Box<[Clause]>) -> SATResult {
    let mut clauses = Clauses::from_dimacs(num_vars, clauses);
    let mut assignments = Assignments::all_unassigned(num_vars);
    let mut assignment_stack: Vec<Vec<usize>> = vec![];
    let mut unassigned: Vec<usize> = (0..num_vars).into_iter().collect();

    loop {
        match unassigned.pop() {
            None => {
                println!("{:?} {:?}", unassigned, assignments);
                return SAT {
                    model: Model {
                        assignments: assignments
                            .assignments
                            .iter()
                            .map(|x| x.to_bool())
                            .collect(),
                    },
                };
            }
            Some(next) => {
                let result = bcp(&mut clauses, &mut assignments, index_to_positive_lit(next));
                match result {
                    Some(lits) => {
                        assignment_stack.push(lits.iter().map(|x| lit_to_index(*x)).collect())
                    }
                    None => match assignment_stack.pop() {
                        Some(mut to_unassign) => {
                            unassigned.append(&mut to_unassign);
                        }
                        None => {
                            return UNSAT;
                        }
                    },
                }
            }
        }
    }
}
