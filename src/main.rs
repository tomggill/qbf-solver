extern crate multimap;

use crate::{parse_config::read_config_json, data_structures::SolverType};

mod dpll;
mod cdcl;
mod parse_config;
mod data_structures;
mod util;
mod resolution;
mod universal_reduction;
mod pure_literal_deletion;
mod literal_selection;
mod tests;

/*
The main function for running the different QBF solver implementations.

Modify config.json to choose your solver configuration and file/benchmark to run.
Run command "cargo run --release"

See README.md for more information.
*/
fn main() {
    let (solver, config) = read_config_json();

    if solver.run_bench {
        if solver.solver_type.eq(&SolverType::DPLL) { dpll::run_bench_directory(solver.path, config, &solver.output) } else { cdcl::run_bench_directory(solver.path, config, &solver.output) }
    } else {
        if solver.solver_type.eq(&SolverType::DPLL) { dpll::run_instance(solver.path, config) } else { cdcl::run_instance(solver.path, config) }
    }
}