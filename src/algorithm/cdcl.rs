use super::interface::*;
use crate::algorithm::interface::Assignment::*;
use crate::algorithm::interface::SATResult::*;
use crate::algorithm::utility::*;
use dimacs::{Clause, Lit, Sign};
use std::collections::{BTreeSet, HashMap, HashSet};
use crate::delta_debug::clause_to_string;

#[derive(Debug)]
pub struct Node {
    level: usize,
    reason: Vec<usize>,
}

struct WatchedClause {
    watched: [Lit; 2],
    clause: Clause,
}

pub enum BcpResult {
    Conflict,
    Implications(Vec<Lit>),
}

struct BCPSolver {
    clauses: Vec<WatchedClause>,
    watches_for_var: Vec<BTreeSet<usize>>,
    assignment: Vec<Assignment>,
    implication_graph: HashMap<usize, Node>,
}

impl BCPSolver {
    pub fn from_dimacs(num_vars: usize, dimacs: &Box<[Clause]>) -> BCPSolver {
        let dimacs = preprocess(&dimacs);
        let mut clauses = Vec::with_capacity(dimacs.len());
        let mut watches_for_var = vec![BTreeSet::new(); num_vars];
        let assignment = vec![Unassigned; num_vars];

        for (index, clause) in (*dimacs).into_iter().enumerate() {
            // if 3 != clause
            //     .lits()
            //     .iter()
            //     .map(|l| lit_to_index(*l))
            //     .collect::<HashSet<usize>>()
            //     .len()
            // {
            //     panic!("fail");
            // }

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

        BCPSolver {
            clauses,
            watches_for_var,
            assignment,
            implication_graph: HashMap::new(),
        }
    }

    fn bcp(&mut self, lit: Lit, level: usize) -> BcpResult {
        self.implication_graph.insert(
            lit_to_index(lit),
            Node {
                level,
                reason: vec![],
            },
        );

        let mut assigned_lits: Vec<Lit> = vec![lit];

        let mut iterations = 0;
        while iterations < assigned_lits.len() {
            let current_lit = assigned_lits[iterations];
            let current_lit_idx = lit_to_index(current_lit);

            self.assignment[current_lit_idx] = assignment_from_sign(current_lit.sign());

            for clause_index in self.watches_for_var[current_lit_idx].clone() {
                let watched_clause = &self.clauses[clause_index];

                let watched1 = self.truth_value_of_literal(watched_clause.watched[0]);
                let watched2 = self.truth_value_of_literal(watched_clause.watched[1]);

                match (watched1, watched2) {
                    (Top, _) | (_, Top) => continue,
                    (Bot, _) | (_, Bot) => {
                        let bot_watch_idx = if watched1 == Bot { 0 } else { 1 };
                        let other_watch_idx = 1 - bot_watch_idx;

                        let new_watched_lit = self.find_next_watched_literal(
                            &watched_clause.clause,
                            watched_clause.watched[other_watch_idx],
                        );
                        match new_watched_lit {
                            Some(found_lit) => {
                                let mut_clause_watcher = &mut self.clauses[clause_index];
                                self.watches_for_var
                                    [lit_to_index(mut_clause_watcher.watched[bot_watch_idx])]
                                .remove(&clause_index);
                                self.watches_for_var[lit_to_index(found_lit)].insert(clause_index);
                                mut_clause_watcher.watched[bot_watch_idx] = found_lit;
                            }
                            None => {
                                let new_unit_lit = watched_clause.watched[other_watch_idx];

                                if self.truth_value_of_literal(new_unit_lit) == Bot {
                                    self.learn_clause_from_conflict(clause_index, level);
                                    self.unassign(&assigned_lits);
                                    return BcpResult::Conflict;
                                } else if self.truth_value_of_literal(new_unit_lit) == Top {
                                    continue;
                                }

                                let reasons = watched_clause
                                    .clause
                                    .lits()
                                    .iter()
                                    .filter(|l| **l != new_unit_lit)
                                    .map(|l| lit_to_index(*l))
                                    .collect();

                                self.implication_graph.insert(
                                    lit_to_index(new_unit_lit),
                                    Node {
                                        level,
                                        reason: reasons,
                                    },
                                );
                                assigned_lits.push(new_unit_lit);
                            }
                        }
                    }
                    (_, _) => panic!(
                        "Should not happen: {:?} {:?}",
                        watched_clause.watched, current_lit
                    ),
                }
            }
            iterations += 1;
        }
        return BcpResult::Implications(assigned_lits);
    }

    fn next_unassigned_lit(&self) -> Option<Lit> {
        match self.assignment.iter().position(|&x| x == Unassigned) {
            Some(idx) => Some(Lit::from_i64((idx + 1) as i64)),
            None => None,
        }
    }

    fn unassign(&mut self, lits: &[Lit]) {
        for lit in lits {
            self.assignment[lit_to_index(*lit)] = Unassigned;
            self.implication_graph.remove(&lit_to_index(*lit));
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
            if (self.assignment[lit_to_index(*lit)] == Unassigned && *lit != other_watched)
                || self.truth_value_of_literal(*lit) == Top
            {
                return Some(*lit);
            }
        }

        return None;
    }

    fn learn_clause_from_conflict(&mut self, clause_index: usize, level: usize) {
        let mut fringe = BTreeSet::new();
        let mut cut = BTreeSet::new();

        for lit in self.clauses[clause_index].clause.lits() {
            fringe.insert(lit_to_index(*lit));
        }
        let mut something_has_been_learnt = false;

        while let Some(&current) = fringe.iter().next() {
            fringe.take(&current);

            let node = self.implication_graph.get(&current).unwrap();
            //Node is UID or not of the same decision level
            // if node.level != level || node.reason.is_empty() {
            if node.reason.is_empty() {
                cut.insert(current);
            } else {
                for lit in &node.reason {
                    fringe.insert(*lit);
                    something_has_been_learnt = true;
                }
            }
        }
        //if !something_has_been_learnt {return;}

        let learned_lits: Vec<Lit> = cut
            .into_iter()
            .map(|index| negate(index_to_lit(index, self.assignment[index])))
            .collect();
        let clause = Clause::from_vec(learned_lits);
        println!("{}",clause_to_string(&clause));
        self.add_watched_clause(clause);
    }

    fn add_watched_clause(&mut self, clause: Clause) {

        let watch1 = clause.lits()[0];

        let watch2;

        match self.find_next_watched_literal(&clause, watch1) {
            Some(other) => {
                watch2 = other;
            }
            None => {
                watch2 = if clause.lits().len() > 1 {
                    clause.lits()[1]
                } else {
                    clause.lits()[0]
                };
            }
        }

        let watched_clause = WatchedClause {
            watched: [watch1, watch2],
            clause: clause.clone(),
        };

        let index = self.clauses.len();
        self.clauses.push(watched_clause);
        self.watches_for_var[lit_to_index(watch1)].insert(index);
        self.watches_for_var[lit_to_index(watch2)].insert(index);
    }
    fn cdcl_recursive(&mut self, level: usize) -> SATResult {
        match self.next_unassigned_lit() {
            None => SAT {
                model: Model {
                    assignments: self.assignment.iter().map(|x| x.to_bool()).collect(),
                },
            },
            Some(next_lit) => {
                for lit in [next_lit, negate(next_lit)] {
                    match self.bcp(lit, level) {
                        BcpResult::Implications(assigned) => match self.cdcl_recursive(level + 1) {
                            SAT { model } => return SAT { model },
                            UNSAT => self.unassign(&assigned),
                        },
                        BcpResult::Conflict => {}
                    }
                }
                UNSAT
            }
        }
    }
}

pub fn cdcl_algorithm(num_vars: usize, clauses: &Box<[Clause]>) -> SATResult {
    let mut solver = BCPSolver::from_dimacs(num_vars, clauses);
    solver.cdcl_recursive(0)
}
