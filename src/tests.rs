#[cfg(test)]
mod tests {
    use std::{fs, io};
    use std::path::{Path, PathBuf};
    use crate::algorithm::cdcl::cdcl_algorithm;
    use crate::algorithm::dpll::dpll_algorithm;
    use crate::algorithm::interface::SATResult::*;
    use crate::algorithm::simple::simple_algorithm;
    use crate::algorithm::utility::{check_result, read_file};

    #[test]
    fn test_dpll() {
        let path = Path::new("test-formulas/short").to_path_buf();

        for (file,path) in get_files_in_dir(path).unwrap(){
            let (num_vars, clauses) = read_file(file.as_str());
            println!("Testing file: {:?}", path);
            let dpll_result = dpll_algorithm(num_vars, &clauses);
            let simple_result = simple_algorithm(num_vars, &clauses);

            match &dpll_result {
                SAT { model } => assert!(check_result(&clauses,&model.assignments)),
                UNSAT => {}
            }

            assert!(dpll_result == simple_result)
        }
    }

    #[test]
    fn test_simple() {
        let path = Path::new("test-formulas/short").to_path_buf();

        for (file, _) in get_files_in_dir(path).unwrap(){
            let (num_vars, clauses) = read_file(file.as_str());
            let simple_result = simple_algorithm(num_vars, &clauses);
            match simple_result
            {
                SAT{ model } =>assert!(check_result(&clauses, &model.assignments)),
                UNSAT =>()
            }
        }
    }

    #[test]
    fn test_cdcl() {
        let path = Path::new("test-formulas/short").to_path_buf();

        for (file,path) in get_files_in_dir(path).unwrap(){
            let (num_vars, clauses) = read_file(file.as_str());
            println!("Testing file: {:?}", path);
            let dpll_result = cdcl_algorithm(num_vars, &clauses);
            let simple_result = simple_algorithm(num_vars, &clauses);

            match &dpll_result {
                SAT {  model } => assert!(check_result(&clauses,&model.assignments)),
                UNSAT => {}
            }

            assert!(dpll_result == simple_result)
        }
    }


    fn get_files_in_dir(dir: PathBuf) -> io::Result<Vec<(String,PathBuf)>> {
        let mut file_contents = Vec::new();

        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let sub_contents = get_files_in_dir(path)?;
                file_contents.extend(sub_contents);
            }
        }
        else if dir.is_file() {
            let content = fs::read_to_string(&dir)?;
            file_contents.push((content,dir));
        }

        Ok(file_contents)
    }
}