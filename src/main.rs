use std::{fs::read, str::from_utf8};

use algorithm::{interface::SATResult, simple::simple_algorithm};
use cli::{parse_cli_args, AlgorithmType};
use dimacs::{parse_dimacs, Instance};
use crate::algorithm::dpll::dpll_algorithm;
use crate::algorithm::cdcl::cdcl_algorithm;

mod algorithm;
mod cli;

fn main() {
    let args = parse_cli_args();

    // Read cnf file as UTF-8
    let file_bytes = read(&args.cnf).unwrap();
    let file_string = from_utf8(&file_bytes).unwrap();

    // Parse CNF input
    let instance = parse_dimacs(file_string).unwrap();
    let (num_vars, clauses) = match instance {
        Instance::Cnf { num_vars, clauses } => (num_vars, clauses),
        Instance::Sat {
            num_vars: _,
            extensions: _,
            formula: _,
        } => panic!("SAT files are not supported, only DIMACS files"),
    };

    // Execute solving algorithm
    let sat_result = match args.algorithm {
        AlgorithmType::Simple => simple_algorithm(num_vars, clauses),
        AlgorithmType::DPLL => dpll_algorithm(num_vars as usize, clauses),
        AlgorithmType::CDCL => cdcl_algorithm(num_vars as usize, clauses),
    };

    // Print solving result
    match sat_result {
        SATResult::UNSAT => {
            println!("Formula is UNSAT");
        }
        SATResult::SAT { model } => {
            println!("Formula is SAT");
            for (var, assignment) in model.assignments.iter().enumerate() {
                println!("{0}: {1}", var, assignment);
            }
        }
    }
}
