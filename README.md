# sat-solver

## Installation

Set up a recent rust installation and then build it:

```bash
cargo build
```
 
## Running

Run it by providing a CNF file in DIMACS format

```bash
cargo run -- -a simple--cnf test-files/test1.cnf
cargo run -- -a dpll--cnf test-files/test1.cnf
cargo run -- -a cdcl--cnf test-files/test1.cnf
```
