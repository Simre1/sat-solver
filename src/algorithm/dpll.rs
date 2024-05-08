use crate::algorithm::interface::Assignment::*;
use crate::algorithm::interface::SATResult::*;
use dimacs::{Clause, Lit, Sign};
use std::collections::BTreeSet;
use super::interface::*;

pub struct WatchedClause {
    watched: [Lit; 2],
    clause: Clause,
}

pub struct DpllSolver {
    clauses: Vec<WatchedClause>,
    watches_for_var: Vec<BTreeSet<usize>>,
    assignment: Vec<Assignment>,
}

impl DpllSolver {
    fn from_dimacs(num_vars: usize, dimacs: Box<[Clause]>) -> DpllSolver {
        let mut clauses = Vec::with_capacity(dimacs.len());
        let mut watches_for_var = vec![BTreeSet::new(); num_vars];
        let assignment = vec![Unassigned; num_vars];

        for (index, clause) in (*dimacs).into_iter().enumerate() {
            let watch1 = clause.lits()[0];
            let watch2 = if clause.lits().len() > 1 {
                clause.lits()[1]
            } else {
                clause.lits()[0]
            };

            clauses.push(WatchedClause {
                watched: [watch1, watch2],
                clause: clause.clone(),
            });

            watches_for_var[lit_to_index(watch1)].insert(index);
            watches_for_var[lit_to_index(watch2)].insert(index);
        }

        DpllSolver {
            clauses,
            watches_for_var,
            assignment,
        }
    }

    fn bcp(&mut self, lit: Lit) -> Option<Vec<Lit>> {
        let mut assigned_lits: Vec<Lit> = vec![lit];

        let mut iterations = 0;
        while iterations < assigned_lits.len() {
            let current_lit = assigned_lits[iterations];
            let current_lit_idx = lit_to_index(current_lit);
            self.assignment[current_lit_idx] = assignment_from_sign(current_lit.sign());

            for clause_index in self.watches_for_var[current_lit_idx].clone() {
                let clause_watcher = & self.clauses[clause_index];

                let watched1 = self.truth_value_of_literal(clause_watcher.watched[0]);
                let watched2 = self.truth_value_of_literal(clause_watcher.watched[1]);

                match (watched1, watched2) {
                    (Top, _) | (_, Top) => continue,
                    (Bot, Bot) => {
                        self.unassign(&assigned_lits);
                        return None;
                    }
                    (Bot, _) | (_, Bot) => {
                        let bot_watch_idx = if watched1 == Bot { 0 } else { 1 };
                        let other_watch_idx = 1 - bot_watch_idx;

                        let new_watched_lit = self.find_next_watched_literal(
                            &clause_watcher.clause,
                            clause_watcher.watched[other_watch_idx],
                        );
                        match new_watched_lit {
                            Some(found_lit) => {
                                let mut_clause_watcher = &mut self.clauses[clause_index];
                                mut_clause_watcher.watched[bot_watch_idx] = found_lit;
                                self.watches_for_var[current_lit_idx].remove(&clause_index);
                                self.watches_for_var[lit_to_index(found_lit)].insert(clause_index);
                            }
                            None => {
                                assigned_lits.push(clause_watcher.watched[other_watch_idx]);
                            }
                        }
                    }
                    (_, _) => panic!("Should not happen"),
                }
            }
            iterations += 1;
        }
        return Some(assigned_lits);
    }

    fn dpll_recursive(&mut self) -> SATResult {
        match self.next_unassigned_lit() {
            None => {
                return SAT {
                    model: Model {
                        assignments: self.assignment.iter().map(|x| x.to_bool()).collect(),
                    },
                };
            }
            Some(next_lit) => {
                for lit in [next_lit, negate(next_lit)] {
                    match self.bcp(lit) {
                        Some(assigned) => match self.dpll_recursive() {
                            SAT { model } => return SAT { model },
                            UNSAT => self.unassign(&assigned),
                        },
                        None => (),
                    }
                }
                return UNSAT;
            }
        }
    }

    fn next_unassigned_lit(&self) -> Option<Lit> {
        match self.assignment.iter().position(|&x| x == Unassigned) {
            Some(idx) => Some(Lit::from_i64((idx + 1) as i64)),
            None => None,
        }
    }

    fn unassign(&mut self, lits: &Vec<Lit>) {
        for lit in lits {
            self.assignment[lit_to_index(*lit)] = Unassigned;
        }
    }

    fn truth_value_of_literal(&self, lit: Lit) -> Assignment {
        let assignment = self.assignment[lit_to_index(lit)];
        match (lit.sign(), assignment) {
            (_, Unassigned) => Unassigned,
            (Sign::Neg, Bot) | (Sign::Pos, Top) => Top,
            (Sign::Pos, Bot) | (Sign::Neg, Top) => Bot,
        }
    }

    fn find_next_watched_literal(&self, clause: &Clause, other_watched: Lit) -> Option<Lit> {
        for lit in clause.lits() {
            if self.assignment[lit_to_index(*lit)] == Unassigned && *lit != other_watched {
                return Some(*lit);
            }
        }

        for lit in clause.lits() {
            if self.truth_value_of_literal(*lit) == Top {
                return Some(*lit);
            }
        }
       
        return None;
    }
}

pub fn dpll_algorithm(num_vars: usize, clauses: Box<[Clause]>) -> SATResult {
    let mut solver = DpllSolver::from_dimacs(num_vars, clauses);
    solver.dpll_recursive()
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