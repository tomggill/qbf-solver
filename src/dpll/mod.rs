mod preprocess;
mod unit_propagate;
mod dpll;
mod bench;
mod dpll_tests;

use crate::{dpll::{preprocess::preprocess, dpll::{dpll, Result}, bench::{run_clause_variable_ratio_instances, run_bench_group}}, data_structures::{Matrix, Statistics, Config}, resolution::pre_resolution};
use std::time::Instant;

/*
A function to run pre-processing, pre-resolution, and dpll, checking for satisfiability and unsatisfiability.
*/
pub fn run_instance(filename: String, config: Config) {
    let timer = Instant::now();
    let matrix = &mut Matrix::new(filename, config);
    let statistics = &mut Statistics::new();
    if matrix.config.pre_process_enabled() { preprocess(matrix, statistics, timer) };
    if matrix.config.pre_resolution_enabled() { pre_resolution(matrix, &mut Vec::new()) };
    let result = dpll(matrix, None, statistics, timer);
    match &result {
        Result::UNSAT => println!("Unsatisfiable"),
        Result::SAT => println!("Satisfiable"),
        Result::Timeout => println!("Runtime has timed out - > 30 seconds.")
    }
}

/*
A function to perform tests on a given set of benchmarks in QDIMACS format. 
*/
pub fn run_bench_directory(path: String, config: Config, filename_to_write: &str) {
    run_bench_group(path, config, filename_to_write);
}

/*
A function to perform tests on a subset of the tacchella data set to measure CPU time
as clause/variable ratio increases.
*/
#[allow(dead_code)]
pub fn run_clause_variable_ratio_bench_directory(config: Config, filename_to_write: &str) {
    run_clause_variable_ratio_instances(config, filename_to_write);
}