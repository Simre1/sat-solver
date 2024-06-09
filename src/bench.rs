use std::fs::{self};
use std::hint::black_box;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use algorithm::simple::simple_algorithm;

use crate::algorithm::cdcl::cdcl_algorithm;
use crate::algorithm::dpll::dpll_algorithm;
use crate::algorithm::utility::read_file;

mod algorithm;
mod cli;
mod tests;
#[tokio::main]
async fn main() {
    let short = Path::new("test-formulas/short").to_path_buf();
    let long = Path::new("test-formulas/long").to_path_buf();

    let files = vec![
        get_files_in_dir(short).unwrap(),
        get_files_in_dir(long).unwrap(),
    ]
    .concat();

    println!("formula simple dpll cdcl");

    for (file, path) in files {
        async {
            let (num_vars, clauses1) = read_file(file.as_str());
            let clauses2 = clauses1.clone();
            let clauses3 = clauses2.clone();

            let simple_result = bench_function(move || {
                simple_algorithm(num_vars, &clauses1);
            })
            .await;
            let dpll_result = bench_function(move || {
                dpll_algorithm(num_vars, &clauses2);
            })
            .await;
            let cdcl_result = bench_function(move || {
                cdcl_algorithm(num_vars, &clauses3);
            })
            .await;

            println!(
                "{:?} {} {} {}",
                path, simple_result, dpll_result, cdcl_result
            );
        }
        .await;
    }
}

async fn bench_function<F>(f: F) -> f64
where
    F: FnOnce() + Send + 'static,
{
    match tokio::time::timeout(
        std::time::Duration::from_secs(1),
        tokio::task::spawn_blocking(move || {
            let now = Instant::now();
            black_box(f());
            let elapsed = now.elapsed();
            return elapsed.as_secs_f64();
        }),
    )
    .await
    {
        Ok(elapsed) => return elapsed.unwrap(),
        Err(_) => -1.,
    }
}

fn get_files_in_dir(dir: PathBuf) -> io::Result<Vec<(String, PathBuf)>> {
    let mut file_contents = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let sub_contents = get_files_in_dir(path)?;
            file_contents.extend(sub_contents);
        }
    } else if dir.is_file() {
        let content = fs::read_to_string(&dir)?;
        file_contents.push((content, dir));
    }

    Ok(file_contents)
}
