#[cfg(test)]
mod test {
    use std::time::Instant;


    use crate::{cdcl::{preprocess::preprocess, cdcl::{cdcl, Result}}, data_structures::{CDCLMatrix, ResolutionConfig, LiteralSelection, Config, Statistics}, resolution::pre_resolution};
    
    fn config() -> Config {
        Config {
            literal_selection: LiteralSelection::VariableStateSum,
            pre_resolution: (false, ResolutionConfig {
                min_ratio: 0.25,
                max_ratio: 0.5,
                max_clause_length: usize::MAX,
                repeat_above: 3,
                iterations: 1,
            }),
            pre_process: true,
            universal_reduction: true,
            pure_literal_deletion: true,
            restarts: true,
        }
    }

    fn timer() -> Instant {
        Instant::now()
    }

    fn run_instance(filename: String) -> Result {
        let matrix = &mut CDCLMatrix::new(filename, config());
        let statistics = &mut Statistics::new();
        let timer = timer();
        if matrix.core_data.config.pre_process_enabled() { preprocess(matrix, statistics, timer); };
        if matrix.core_data.config.pre_resolution_enabled() { pre_resolution(&mut matrix.core_data, &mut matrix.original_clause_list) };
        let (_invariant, _backtrack_level, result) = cdcl(matrix, None, statistics, timer);
        return result;
    }
    
    /* START OF GENERAL INSTANCE TESTS */
    /* Note: These have been reduced in scope for submission */

    #[test]
    fn test_instance_1() {
        let filename = "./benchmarks/samples/example.qdimacs".to_string();
        let result = run_instance(filename);
        assert_eq!(Result::SAT, result);
    }
    
    /* END OF GENERAL INSTANCE TESTS */
}