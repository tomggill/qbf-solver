use std::{fs, time::{Duration, Instant}, collections::{HashMap, BTreeMap}};
use multimap::MultiMap;
use regex::Regex;

use crate::{cdcl::{preprocess::preprocess, cdcl::{cdcl, Result}}, data_structures::{CDCLMatrix, Config, Statistics}, resolution::pre_resolution, util::read_instance_name};

/*
A function to run a directory of files in QDIMACS format. 
It will run each problem with an automatic timeout at 30 seconds.

Stores detailed results in a file with the provided name "results-<filename_to_write>".
*/
pub fn run_bench_group(group: String, config: Config, filename_to_write: &str) {
    let mut test_times = BTreeMap::new();
    let paths = fs::read_dir(&group).unwrap();
    let (mut total, mut satisfiable, mut unsatisfiable, mut timeout) = (0, 0, 0, 0);
    let bench_timer = Instant::now();
    let mut statistic_database : HashMap<String, (i32,i32,i32, Result)> = HashMap::new();
    for path in paths {
        let instance_timer = Instant::now();
        let file_path = path.unwrap().path().display().to_string();
        
        let matrix = &mut CDCLMatrix::new(file_path.clone(), config.clone());
        let instance_name = read_instance_name(&file_path);
        let statistics = &mut Statistics::new();
        if matrix.core_data.config.pre_process_enabled() { preprocess(matrix, statistics, instance_timer) };
        if matrix.core_data.config.pre_resolution_enabled() { pre_resolution(&mut matrix.core_data, &mut matrix.original_clause_list) };
        let (_invariant, _backtrack_level, result) = cdcl(matrix, None, statistics, instance_timer);
        test_times.insert(instance_name.clone(), instance_timer.elapsed());
        statistic_database.insert(instance_name, (statistics.propagation_count, statistics.backtrack_count, statistics.learned_clause_count, result.clone()));
        total += 1;
        match &result {
            Result::UNSAT => unsatisfiable += 1,
            Result::SAT => satisfiable += 1,
            Result::Timeout => timeout += 1,
            Result::Restart => println!("ERROR WITH RESTARTS"),
        }
    }
    // Formatting to store overall results
    let mut output_string = format!("--- CDCL --- \nCONFIG: [Literal Selection: {:?}, Pre-Resolution: {}, Pre-Process: {}, Universal Reduction: {}, Pure Literal Deletion: {}]", 
                                            config.literal_selection, config.pre_resolution.0, config.pre_process, config.universal_reduction, config.pure_literal_deletion);
    if config.pre_resolution_enabled() {
        output_string += &format!("\nPre-Resolution Config: [min_ratio: {}, max_ratio: {}, max_clause_length: {}, repeat_above: {}, iterations: {}]", config.pre_resolution.1.min_ratio, config.pre_resolution.1.max_ratio, config.pre_resolution.1.max_clause_length, config.pre_resolution.1.repeat_above, config.pre_resolution.1.iterations);
    }
    output_string += &format!("\n--------------------------------------------------------------\nTotal: {}, Sat: {}, Unsat: {}, Timeout: {}\nComplete time: {:?}", total, 
                                satisfiable, unsatisfiable, timeout, bench_timer.elapsed());
    for (key, val) in test_times {
        let stats = statistic_database.get(&key).unwrap();
        output_string += &format!("\nInstance: {} -- Runtime: {:?} -- Result: {:?}  -- Propagations: {}, Backtracks: {}, Learned Clauses: {}", key, val, stats.3, stats.0, stats.1, stats.2);
    }
    let pathname = format!("output-{}", filename_to_write);
    fs::write(pathname, output_string).expect("Unable to write file");
}


/*
A function to run the Tacchella data set suite. I've decided to separate this benchmark as I wanted to gather 
separate information from other benchmarks. This function is not necessary for general usage of the solvers.

Stores detailed results in a file with the provided name "results-<filename_to_write>".
*/
pub fn run_clause_variable_ratio_instances(config: Config, filename_to_write: &str) {
    let paths = fs::read_dir("./benchmarks/tacchella").unwrap();
    let mut output = MultiMap::new();
    for path in paths {
        let timer = Instant::now();
        let file_path = path.unwrap().path().display().to_string();
        let problem_setup = read_clause_variable_data(file_path.clone());

        let matrix = &mut CDCLMatrix::new(file_path, config.clone());
        let statistics = &mut Statistics::new();
        if matrix.core_data.config.pre_process_enabled() { preprocess(matrix, statistics, timer) };
        if matrix.core_data.config.pre_resolution_enabled() { pre_resolution(&mut matrix.core_data, &mut matrix.original_clause_list) };
        let (_invariant, _backtrack_level, result) = cdcl(matrix, None, statistics, timer);
        match &result {
            Result::UNSAT | Result::SAT | Result::Timeout => output.insert(problem_setup, timer.elapsed()),
            Result::Restart => println!("Error occurred with restart functionality."),
        }
    }
    let mut ratios = MultiMap::new();
    let mut output_string = format!("------ CDCL ------ \n(<quantifier alternation number>, <variable number>, <clause number>): <average time per solved instance>");
    for (key, value) in output {
        ratios.insert((key.1, key.2), value.iter().sum::<Duration>());
        output_string += &format!("\n({}qbf, {}var, {}cl): {:?}", key.0, key.1, key.2, value.iter().sum::<Duration>())
    }
    output_string += &format!("\n(<Clause-variable values>) -> Combined time");
    for (key, value) in ratios {
        output_string += &format!("\nSums: ({}, {}) -> {:?}", key.0, key.1, value.iter().sum::<Duration>());
    }
    let pathname = format!("output-{}", filename_to_write);
    fs::write(pathname, output_string).expect("Unable to write file");
}

/*
The tacchella instance set is built on the size of instances and they explicitly note the number of variables and
clauses within an instance. I use this to extract evaluation data on the effectiveness of my algorithms. 
This function finds this instance setup data within the file name.

Returns [# of qbf alternations, # of variables, # of clauses].
*/
pub fn read_clause_variable_data(file_path: String) -> (i32, i32, i32) {
    let re_separate_data = Regex::new(r"\d+qbf|\d+var|\d+cl").unwrap();
    let instance_setup: Vec<&str> = re_separate_data.find_iter(&file_path).map(|m| m.as_str()).collect();
    let re_find_numbers = Regex::new(r"\d+").unwrap();
    let mut problem_setup = Vec::new();
    for found_match in instance_setup {
        let number = re_find_numbers.find(found_match).map(|m| m.as_str()).unwrap().parse::<i32>().unwrap();
        problem_setup.push(number);
    }
    return (problem_setup[0], problem_setup[1], problem_setup[2]);
}