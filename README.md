# sat-solver

## Installation

Set up a recent rust installation and then build it:

```bash
cargo build
```
 
## Running

Run it by providing a CNF file in DIMACS format and choose one of the algorithms (simple, dpll, cdcl)

```bash
cargo run -- -a simple --cnf test-files/test1.cnf
cargo run -- -a dpll --cnf test-files/test1.cnf
cargo run -- -a cdcl --cnf test-files/test1.cnf
```

## Algorithms

We have implemented:
- A simple recursive algorithm which tries to find a solution in a brute-force manner: [simple.rs](https://github.com/Simre1/sat-solver/blob/main/src/algorithm/simple.rs)
- A dpll algorithm: [dpll.rs](https://github.com/Simre1/sat-solver/blob/main/src/algorithm/dpll.rs)
- A cdcl algorithm (although buggy): [cdcl.rs](https://github.com/Simre1/sat-solver/blob/main/src/algorithm/cdcl.rs)

## Benchmarking

`./bench.sh` will run the the algorithms for each algorithm in the `test-formulas/bench` folder in parallel. 
It uses the `time` and the `timeout` linux utilities to measure the time and stop them at 5 minutes.

`./bench-csv-format.sh` formats the output of `bench.sh` into the csv format in order to easily create diagrams.
