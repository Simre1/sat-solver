use super::interface::*;
use crate::algorithm::interface::Assignment::*;
use crate::algorithm::interface::SATResult::*;
use dimacs::{Clause, Lit, Sign};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug)]
pub struct Node {
    // level: usize,
    reason: Vec<Lit>,
}

struct WatchedClause {
    watched: [Lit; 2],
    clause: Clause,
}

pub enum BcpResult {
    Conflict(usize),
    Implications(Vec<Lit>),
}

struct BCPSolver {
    clauses: Vec<WatchedClause>,
    watches_for_var: Vec<BTreeSet<usize>>,
    assignment: Vec<Assignment>,
    implication_graph: HashMap<Lit, Node>,
}

impl BCPSolver {
    pub fn from_dimacs(num_vars: usize, dimacs: Box<[Clause]>) -> BCPSolver {
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

        BCPSolver {
            clauses,
            watches_for_var,
            assignment,
            implication_graph: HashMap::new(),
        }
    }

    fn bcp(&mut self, lit: Lit) -> BcpResult {
        self.implication_graph.insert(lit, Node { reason: vec![] });

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
                    (Bot, Bot) => {
                        self.unassign(&assigned_lits);
                        return BcpResult::Conflict(clause_index);
                    }
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
                                mut_clause_watcher.watched[bot_watch_idx] = found_lit;
                                self.watches_for_var[current_lit_idx].remove(&clause_index);
                                self.watches_for_var[lit_to_index(found_lit)].insert(clause_index);
                            }
                            None => {
                                let new_unit_lit = watched_clause.watched[other_watch_idx];

                                if self.truth_value_of_literal(new_unit_lit) == Bot {
                                    self.unassign(&assigned_lits);
                                    return BcpResult::Conflict(clause_index);
                                } else if self.truth_value_of_literal(new_unit_lit) == Top {
                                    continue;
                                }

                                let reasons = watched_clause
                                    .clause
                                    .lits()
                                    .iter()
                                    .filter(|l| **l != new_unit_lit)
                                    .map(|l| negate(*l))
                                    .collect();
                                println!("Reasons: {:?}", &reasons);
                                self.implication_graph
                                    .insert(new_unit_lit, Node { reason: reasons });
                                assigned_lits.push(new_unit_lit);
                            }
                        }
                    }
                    (_, _) => panic!("Should not happen"),
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

    fn collect_relevant_decisions(&self, clause_index: usize) -> Vec<Lit> {
        let mut visited = HashSet::new();
        let mut queue = Vec::new();
        let mut decisions = HashSet::new();

        for lit in self.clauses[clause_index].clause.lits() {
            queue.push(negate(*lit));
        }

        while let Some(current) = queue.pop() {
            if (visited.contains(&current)) {
                continue;
            }

            visited.insert(current);

            println!("{:?}", current);
            let mut reasons: Vec<Lit> =
                self.implication_graph.get(&current).unwrap().reason.clone();

            if reasons.len() == 0 {
                decisions.insert(current);
            } else {
                queue.append(&mut reasons);
            }
        }

        return decisions.into_iter().collect();
    }
}

pub fn cdcl_algorithm(num_vars: usize, clauses: Box<[Clause]>) -> SATResult {
    let mut solver = BCPSolver::from_dimacs(num_vars, clauses);
    cdcl_recursive(&mut solver)
}

fn cdcl_recursive(bcp_solver: &mut BCPSolver) -> SATResult {
    match bcp_solver.next_unassigned_lit() {
        None => {
            return SAT {
                model: Model {
                    assignments: bcp_solver.assignment.iter().map(|x| x.to_bool()).collect(),
                },
            };
        }
        Some(next_lit) => {
            for lit in [next_lit, negate(next_lit)] {
                match bcp_solver.bcp(lit) {
                    BcpResult::Implications(assigned) => match cdcl_recursive(bcp_solver) {
                        SAT { model } => return SAT { model },
                        UNSAT => bcp_solver.unassign(&assigned),
                    },
                    BcpResult::Conflict(clause_index) => {
                        for (key, val) in bcp_solver.implication_graph.iter() {
                            println!("{:?}: {:?}", &key, &val);
                        }
                        println!("\n\n {:?}", &bcp_solver.clauses[clause_index].clause,);
                        println!("\n\n {:?}", &bcp_solver.assignment);
                        let decisions: Vec<Lit> =
                            bcp_solver.collect_relevant_decisions(clause_index);
                        let learned_clause =
                            Clause::from_vec(decisions.into_iter().map(|l| negate(l)).collect());

                        let watch1 = learned_clause.lits()[0];

                        let watch2;

                        match bcp_solver.find_next_watched_literal(&learned_clause, watch1) {
                            Some(other) => {
                                watch2 = other;
                            }
                            None => {
                                watch2 = if learned_clause.lits().len() > 1 {
                                    learned_clause.lits()[1]
                                } else {
                                    learned_clause.lits()[0]
                                };
                            }
                        }

                        let watched_clause = WatchedClause {
                            watched: [watch1, watch2],
                            clause: learned_clause.clone(),
                        };

                        let index = bcp_solver.clauses.len();
                        bcp_solver.clauses.push(watched_clause);
                        bcp_solver.watches_for_var[lit_to_index(watch1)].insert(index);
                        bcp_solver.watches_for_var[lit_to_index(watch2)].insert(index);
                    }
                }
            }
            return UNSAT;
        }
    }
}
