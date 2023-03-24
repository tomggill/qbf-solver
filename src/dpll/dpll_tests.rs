#[cfg(test)]
mod test {
    use std::time::Instant;

    use crate::{dpll::{preprocess::preprocess, dpll::{dpll, Result}}, data_structures::{Matrix, ResolutionConfig, LiteralSelection, Config, Statistics}, resolution::pre_resolution};
    
    fn config() -> Config {
        Config {
            literal_selection: LiteralSelection::Ordered,
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
            restarts: false,
        }
    }

    fn timer() -> Instant {
        Instant::now()
    }

    fn run_instance(filename: String) -> Result {
        let matrix = &mut Matrix::new(filename, config());
        let statistics = &mut Statistics::new();
        let timer = timer();
        if matrix.config.pre_process_enabled() { preprocess(matrix, statistics, timer) };
        if matrix.config.pre_resolution_enabled() { pre_resolution(matrix, &mut Vec::new()) };
        return dpll(matrix, None, statistics, timer);
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