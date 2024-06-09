use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use dimacs::{Clause, Lit, Sign};
use crate::algorithm::cdcl::cdcl_algorithm;
use crate::algorithm::interface::SATResult::UNSAT;
use crate::algorithm::utility::read_file;

#[test]
fn reduce_problem(){
    println!("test");
    let file = fs::read_to_string(Path::new("../test-formulas/long/prime289.in").to_path_buf()).unwrap();
    let (num_vars, clauses) = read_file(file.as_str());
    let mut clauses_vec = Vec::from(clauses);

    let test_function = |clauses: &Vec<Clause>| cdcl_algorithm(num_vars, &clauses.clone().into_boxed_slice()) == UNSAT;

    let min = delta_debug(clauses_vec.clone(), test_function);

    let num_clauses= clauses_vec.len();
    let header = format!("p cnf {num_vars} {num_clauses}");
    let content = std::iter::once(header).chain(clauses_vec.iter().map(|c| clause_to_string(c))).collect::<Vec<String>>().join("\n");

    let mut file = File::create("test-files/test10.cnf").unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

fn delta_debug<T, F>(input: Vec<T>, test: F) -> Vec<T>
    where
        T: Clone,
        F: Fn(&Vec<T>) -> bool,
{
    delta_debug_recursive(input, test, 2)
}

fn delta_debug_recursive<T, F>(input: Vec<T>, test: F, n: usize) -> Vec<T>
    where
        T: Clone,
        F: Fn(&Vec<T>) -> bool,
{
    if input.is_empty() || n > input.len() {
        return input;
    }

    let chunk_size = (input.len() + n - 1) / n;
    let mut chunks: Vec<&[T]> = input.chunks(chunk_size).collect();
    let mut reduced_input = input.clone();

    for (i, chunk) in chunks.iter().enumerate() {
        let complement: Vec<T> = input
            .iter()
            .enumerate()
            .filter_map(|(j, item)| if j / chunk_size != i { Some(item.clone()) } else { None })
            .collect();

        if test(&complement) {
            return delta_debug_recursive(complement, test, 2);
        }
    }

    if n < input.len() {
        return delta_debug_recursive(input, test, n * 2);
    }

    reduced_input
}
fn lit_to_string(lit: &Lit) ->String{
    match lit.sign() {
        Sign::Pos => {lit.var().0.to_string()}
        Sign::Neg => {(-(lit.var().0 as isize)).to_string()}
    }
}

pub fn clause_to_string(clause: &Clause) -> String{
    clause.lits().iter()
        .map(lit_to_string)
        .chain(std::iter::once("0".to_string()))
        .collect::<Vec<String>>()
        .join(" ")
}