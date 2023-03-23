mod preprocess;
mod unit_propagate;
mod cdcl;
mod bench;
mod conflict_analysis;
mod cdcl_tests;

use std::time::Instant;
use crate::{cdcl::{preprocess::preprocess, cdcl::{Result, cdcl}, bench::{run_clause_variable_ratio_instances, run_bench_group}}, data_structures::{CDCLMatrix, Statistics, Config}, resolution::pre_resolution};

/*
A function to run pre-processing, pre-resolution, and dpll, checking for satisfiability and unsatisfiability.
*/
pub fn run_instance(filename: String, config: Config) {
    let timer = Instant::now();
    let matrix = &mut CDCLMatrix::new(filename, config);
    let statistics = &mut Statistics::new();
    if matrix.core_data.config.pre_process_enabled() { preprocess(matrix, statistics, timer); };
    if matrix.core_data.config.pre_resolution_enabled() { pre_resolution(&mut matrix.core_data, &mut matrix.original_clause_list) };
    let (_invariant, _backtrack_level, result) = cdcl(matrix, None, statistics, timer);
    match &result {
        Result::UNSAT => println!("Unsatisfiable"),
        Result::SAT => println!("Satisfiable"),
        Result::Timeout => println!("Runtime has timed out: > 30 seconds."),
        Result::Restart => println!("ERROR WITH RESTARTS")
    }
}

/*
A function to perform tests on a given set of benchmarks in qdimacs format.
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