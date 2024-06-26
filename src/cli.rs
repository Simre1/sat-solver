use std::fmt::{self, Display};
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

/// Sat solver CLI args
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CLIArgs {
    /// CNF file in DIMACS format
    #[arg(short, long)]
    pub cnf: PathBuf,

    #[arg(short, long, default_value_t = AlgorithmType::DPLL)]
    /// Algorithm to use
    pub algorithm: AlgorithmType,
}

impl Display for AlgorithmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlgorithmType::Simple => f.write_str("simple"),
            AlgorithmType::DPLL => f.write_str("dpll"),
            AlgorithmType::CDCL => f.write_str("cdcl"),
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub enum AlgorithmType {
    Simple,
    DPLL,
    CDCL,
}

impl FromStr for AlgorithmType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "simple" => Ok(AlgorithmType::Simple),
            "dpll" => Ok(AlgorithmType::DPLL),
            "cdcl" => Ok(AlgorithmType::CDCL),
            _ => Err("Not recognized".to_string()
                + s
                + "\nSupported algorithms: \"simple\", \"dpll\", \"cdcl\""),
        }
    }
}

pub fn parse_cli_args() -> CLIArgs {
    CLIArgs::parse()
}
