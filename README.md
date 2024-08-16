# QBF Solver Evaluator
This tool is a solver for quantified Boolean formulas (QBF) in prenex conjunctive normal form (PCNF) in the [QDIMACS](http://www.qbflib.org/qdimacs.html) format. This tool contains functionality for the Davis-Putnam-Logemann-Loveland (DPLL) and Conflict Driven Clause Learning (CDCL) backtracking algorithms with optimisation techniques. Solver configuration is customisable.

See - [Dissertation](https://github.com/tomggill/qbf-solver/blob/main/Analysing_Optimisation_and_Solving_Techniques_in_QBF_Solvers.pdf)

## Installation
This tool is written in [Rust](https://www.rust-lang.org/). You can download the latest version of the Rust compiler [here](https://rustup.rs/), alternatively you can follow the instructions in the [rust docs](https://doc.rust-lang.org/cargo/getting-started/installation.html). The tool is written as a rust crate so no static binaries are created. To build the tool use ```cargo build --release``` then run the tool using ```cargo run --release```.

## Usage
The input QBF file format should be in [QDIMACS](http://www.qbflib.org/qdimacs.html) file format. The Output is the result Satisfiable or Unsatisfiable when running the solver on an individual instance. When running the solver on a benchmark of instances, a output file is produced containing statistical data and results. No command line paramters are required as the configuration of the solver is determined from the config.json file. 

```json
{
    "RunBenchmark": false,
    "BenchmarkPath": "./benchmarks/samples",
    "InstancePath": "./benchmarks/samples/example.qdimacs",
    "OutputFileName": "instance-results",
    "SolverOptions": {
        "SolverType": "CDCL",
        "LiteralSelection": "VSS",
        "Preprocess": true,
        "UniversalReduction": true,
        "PureLiteralDeletion": true,
        "Restarts": true,
        "PreResolution": false,
        "PreResolutionConfig": {
            "min_ratio": 0.25,
            "max_ratio": 0.5,
            "max_clause_length": "infinity",
            "repeat_above": 3,
            "iterations": 1
        }
    }
}
```

```RunBenchmark```: Determines whether the solver should be run on a directory of QBF instances or a singular QBF instance.

```BenchmarkPath```: The directory path to the folder containing the benchmark instances to be solved.

```InstancePath```: The file path to the instance to be solved.

```OutputFileName```: The name given to the output file containing the results from the execution of the solver on a benchmark.

```SolverType```: The core solving algorithm to be used - either DPLL or CDCL.

```LiteralSelection```: The literal selection method to be used - either VSS or Ordered.

```Preprocess, UniversalReduction, PureLiteralDeletion, Restarts, PreResolution```: Options to determine whether to use the repective optimisation in the solver.

```PreResolutionConfig```: Contains the hyperparamter values used when performing pre-resolution.

```min_ratio, max_ratio```: The lower and upper bound on how many resolved clauses to add to the clause database.

```max_clause_length```: The maximum clause length allowed to be added to the clause database after pre-resolution.

```repeat_above```: Repeats resolution for a given literal if the recently resolved clause is above a certain length.

```iterations```: Determines how many pre-resolution iterations to perform.
