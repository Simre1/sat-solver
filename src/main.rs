use std::fs::read;
use std::str::from_utf8;

use algorithm::{interface::SATResult, simple::simple_algorithm};
use cli::{AlgorithmType, parse_cli_args};

use crate::algorithm::cdcl::cdcl_algorithm;
use crate::algorithm::dpll::dpll_algorithm;
use crate::algorithm::utility::read_file;

mod algorithm;
mod cli;
mod tests;

fn main() {
    let args = parse_cli_args();

    // Read cnf file as UTF-8
    let file_bytes = read(&args.cnf).unwrap();
    let file_string = from_utf8(&file_bytes).unwrap();
    let (num_vars, clauses) = read_file(&file_string);

    // Execute solving algorithm
    let sat_result = match args.algorithm {
        AlgorithmType::Simple => simple_algorithm(num_vars, &clauses),
        AlgorithmType::DPLL => dpll_algorithm(num_vars , &clauses),
        AlgorithmType::CDCL => cdcl_algorithm(num_vars , &clauses),
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


