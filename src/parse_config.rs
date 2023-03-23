use std::fs::File;
use serde_json::Value;

use crate::data_structures::{SolverType, LiteralSelection, Config, ResolutionConfig, Solver};

/*
A function to read the configuration of the solver within config.json.

Returns:
- Solver type
- Whether it's a bench
- Path to bench or instance
- Solver config options
*/
pub fn read_config_json() -> (Solver, Config) {
    let file = File::open("./config.json").unwrap();
    let json: Value = serde_json::from_reader(file).expect("file should be valid JSON");
    let solver_options = json.get("SolverOptions").expect("file should have SolverOptions key");

    let solver_type_json = solver_options.get("SolverType").expect("file should have SolverType key");
    let solver_type = read_solver_type_json(solver_type_json).expect("SolverType should be a valid solver: CDCL or DPLL");

    let run_bench_json = json.get("RunBenchmark").expect("file should have RunBenchmark key");
    let run_bench = read_boolean_json(run_bench_json).expect("RunBenchmark should be a Boolean value");
    let path = read_path(run_bench, &json);

    let output_json = json.get("OutputFileName").expect("file should have OutputFileName key");
    let output = read_string_json(output_json).expect("OutputFileName must be a string");

    let solver = Solver {
        solver_type,
        run_bench,
        path,
        output,
    };

    let pre_resolution_options = solver_options.get("PreResolutionConfig").expect("file should have PreResolutionConfig key");
    let min_ratio_json = pre_resolution_options.get("min_ratio").expect("file should have min_ratio key");
    let max_ratio_json = pre_resolution_options.get("max_ratio").expect("file should have max_ratio key");
    let max_clause_length_json = pre_resolution_options.get("max_clause_length").expect("file should have max_clause_length key");
    let repeat_above_json = pre_resolution_options.get("repeat_above").expect("file should have repeat_above key");
    let iterations_json = pre_resolution_options.get("iterations").expect("file should have iterations key");
    let resolution_config = ResolutionConfig {
        min_ratio: read_number_json_f32(min_ratio_json).expect("min_ratio value must be a valid number or 'infinity'"),
        max_ratio: read_number_json_f32(max_ratio_json).expect("min_ratio value must be a valid number or 'infinity'"),
        max_clause_length: read_number_json_usize(max_clause_length_json).expect("max_clause_length value must be a valid number or 'infinity'"),
        repeat_above: read_number_json_usize(repeat_above_json).expect("repeat_above value must be a valid number or 'infinity'"),
        iterations: read_number_json_i32(iterations_json).expect("iterations value must be a valid number")
    };

    let literal_selection_json = solver_options.get("LiteralSelection").expect("file should have LiteralSelection key");
    let literal_selection = read_literal_selection_json(literal_selection_json).expect("LiteralSelection should be a valid type: VSS or Ordered");

    let pre_process_json = solver_options.get("Preprocess").expect("file should have Preprocess key");
    let pre_process = read_boolean_json(pre_process_json).expect("Preprocess should be a Boolean value");

    let universal_reduction_json = solver_options.get("UniversalReduction").expect("file should have UniversalReduction key");
    let universal_reduction = read_boolean_json(universal_reduction_json).expect("UniversalReduction should be a Boolean value");

    let pure_literal_deletion_json = solver_options.get("PureLiteralDeletion").expect("file should have PureLiteralDeletion key");
    let pure_literal_deletion = read_boolean_json(pure_literal_deletion_json).expect("PureLiteralDeletion should be a Boolean value");

    let restarts_json = solver_options.get("Restarts").expect("file should have Restarts key");
    let restarts = read_boolean_json(restarts_json).expect("Restarts should be a Boolean value");

    let pre_resolution_json = solver_options.get("PreResolution").expect("file should have PreResolution key");
    let pre_resolution = (read_boolean_json(pre_resolution_json).expect("PreResolution should be a Boolean value"), resolution_config);


    let config = Config {
        literal_selection,
        pre_resolution,
        pre_process,
        universal_reduction,
        pure_literal_deletion,
        restarts,
    };

    return (solver, config);
}

/*
A function to read float numbers from json. Returns float value or None if invalid.
*/
pub fn read_number_json_f32(value: &Value) -> Option<f32> {
    if value.is_number() {
        return Some(value.as_f64().unwrap() as f32);
    } else if value.is_string() {
        if value.as_str().unwrap().to_lowercase().eq("infinity") {
            return Some(f32::MAX);
        }
    }
    return None;
}

/*
A function to read usize numbers from json. Returns usize value or None if invalid.
*/
pub fn read_number_json_usize(value: &Value) -> Option<usize> {
    if value.is_number()  && !value.is_f64() {
        return Some(value.as_u64().unwrap() as usize);
    } else if value.is_string() {
        if value.as_str().unwrap().to_lowercase().eq("infinity") {
            return Some(usize::MAX);
        }
    }
    return None
}

/*
A function  to read a integer numbers from json. Returns integer value or None if invalid.
*/
pub fn read_number_json_i32(value: &Value) -> Option<i32> {
    if value.is_number() && !value.is_f64() {
        return Some(value.as_i64().unwrap() as i32);
    }
    return None;
}

/*
A function to read SolverType objects from json. Returns SolverType object or None if invalid.
*/
pub fn read_solver_type_json(value: &Value) -> Option<SolverType> {
    if value.is_string() {
        if value.as_str().unwrap().to_lowercase().eq("cdcl") {
            return Some(SolverType::CDCL);
        } else if value.as_str().unwrap().to_lowercase().eq("dpll") {
            return Some(SolverType::DPLL);
        }
    }
    return None;
}

/*
A function to read LiteralSelection objects from json. Returns LiteralSelection object or None if invalid.
*/
pub fn read_literal_selection_json(value: &Value) -> Option<LiteralSelection> {
    if value.is_string() {
        if value.as_str().unwrap().to_lowercase().eq("vss") {
            return Some(LiteralSelection::VariableStateSum);
        } else if value.as_str().unwrap().to_lowercase().eq("ordered") {
            return Some(LiteralSelection::Ordered);
        }
    }
    return None;
}

/*
A function to read Boolean values from json. Returns Boolean value or None if invalid.
*/
pub fn read_boolean_json(value: &Value) -> Option<bool> {
    if value.is_boolean() {
        return value.as_bool();
    } else {
        return None;
    }
}

/*
A function to read path strings from json. Returns path as String.
*/
pub fn read_path(run_bench: bool, json: &Value) -> String {
    let path_json: &Value;
    if run_bench {
        path_json = json.get("BenchmarkPath").expect("file should have BenchmarkPath key");
    } else {
        path_json = json.get("InstancePath").expect("file should have InstancePath key");
    }
    let path = read_string_json(path_json).expect("BenchmarkPath and InstancePath must be a string");
    return path;
}

/*
A function to read String values from json. Returns String value or None if invalid.
*/
pub fn read_string_json(value: &Value) -> Option<String> {
    if value.is_string() {
        return Some(value.as_str().unwrap().to_string());
    }
    return None;
}