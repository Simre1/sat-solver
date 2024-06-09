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
mod delta_debug;

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
            let res = model.assignments.iter()
                .enumerate()
                .map(|(idx, val)| if *val {idx as isize +1} else {-(idx as isize+1)})
                .map(|lit|lit.to_string())
                .collect::<Vec<String>>()
                .join(" ");

            println!("{}", res);
        }
    }
}


