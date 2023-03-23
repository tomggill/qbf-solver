use std::collections::HashMap;
use std::time::Instant;
use multimap::MultiMap;

use crate::{cdcl::{unit_propagate::unit_propagate, conflict_analysis::analyse_conflict, preprocess::preprocess}, data_structures::{CDCLMatrix, Clause, QuantifierType, ClauseSet, Quantifier, Assignment, Statistics, LiteralSelection}, literal_selection::{select_literal_vss, select_literal}};

/*
A struct to store the result of the CDCL procedure.

SAT => Satisfiable at current decision branch.
UNSAT => Unsatisfiable at current decision branch.
Timeout => Instance timeout, stop running the current instance.
Restart => Indicates that a restart should be performed, return to top of the decision tree.
*/
#[derive(Clone, Debug, PartialEq)]
pub enum Result {
    SAT,
    UNSAT,
    Timeout,
    Restart,
}

/*
A function that will perform the Conflict Driven Clause Learning (CDCL) algorithm with a selection of optimisations
from the set {Universal Reduction, Pre-Resolution (done prior), Pre-Process (done prior)}.
Has one of the literal selection schemes {Ordered, Variable State Sum}.

Returns SAT (satisfiable), UNSAT (unsatisfiable), Timeout, or Restart.
*/
pub fn cdcl(matrix: &mut CDCLMatrix, decision_branch: Option<i32>, statistics: &mut Statistics, timer: Instant) -> (Clause, i32, Result) {
    loop {
        if timer.elapsed().as_secs() > 30 {
            return timeout();
        }
        if !decision_branch.is_none() {
            unit_propagate(matrix, vec![decision_branch.unwrap()], true, statistics);
        }
        if matrix.core_data.clause_set.contains_empty_set() { // Current assignment is satisfiable.
            return satisfiable();
        } else if matrix.core_data.clause_set.contains_empty_clause() { // Current assignment is unsatisfiable.
            if matrix.core_data.config.restarts_enabled() && matrix.restart_data.should_restart() {
                return perform_restart(matrix);
            }
            // Analyse conflict here.
            let (learned_clause, backtrack_level) = analyse_conflict(matrix, statistics);
            if !learned_clause.is_empty() && matrix.core_data.config.restarts_enabled() {matrix.restart_data.increment_current_conflicts()};
            return (learned_clause, backtrack_level, Result::UNSAT);
        }
        let pre_selection_quantifier_list = matrix.core_data.quantifier_list.clone();

        let (literal, quantifier_type) = if matrix.core_data.config.literal_selection.eq(&LiteralSelection::Ordered) 
                                                        {select_literal(&mut matrix.core_data)} else {select_literal_vss(&mut matrix.core_data)};

        matrix.increment_decision_level();
        // Necessary copying of data as they are all edited and propagated back up with edited data.
        let stored_structures = cache_necessary_structures(matrix);

        let (learned_clause, backtrack_level, result) = cdcl(matrix, Some(literal), statistics, timer);

        restore_necessary_structures(matrix, stored_structures);

        match (&result, &quantifier_type) {
            (Result::UNSAT, QuantifierType::Universal) | (Result::UNSAT, QuantifierType::Existential) => {
                if backtrack_level == matrix.decision_level {
                    if learned_clause.is_empty() {
                        if quantifier_type.eq(&QuantifierType::Universal) {
                            return (learned_clause, backtrack_level - 1, result);
                        } else {
                            matrix.decision_level -= 1;
                            statistics.increment_backtrack_count();
                            return cdcl(matrix, Some(-literal), statistics, timer);
                        }
                    }
                    statistics.increment_backtrack_count();
                    matrix.core_data.quantifier_list = pre_selection_quantifier_list;
                    matrix.decision_level -= 1;
                    matrix.add_clause(&learned_clause); // Adding new learned clause
                    continue;
                } else if !learned_clause.is_unit_clause().is_none() && matrix.decision_level == 1 {
                    // Conflict analysis returns backtrack_level 0 for unit clauses.
                    statistics.increment_backtrack_count();
                    matrix.add_clause(&learned_clause);
                    matrix.core_data.quantifier_list = pre_selection_quantifier_list;
                    matrix.decision_level -= 1;
                    preprocess(matrix, statistics, timer); // Simplify problem permanently.
                    if matrix.core_data.clause_set.contains_empty_set() {
                        return satisfiable();
                    } else if matrix.core_data.clause_set.contains_empty_clause() {
                        return unsatisfiable();
                    } else {
                        continue;
                    }
                } else {
                    return (learned_clause, backtrack_level, result);
                }
            },
            (Result::SAT, QuantifierType::Universal) => {
                matrix.decision_level -= 1;
                statistics.increment_backtrack_count();
                return cdcl(matrix, Some(-literal), statistics, timer);
            },
            (Result::SAT, QuantifierType::Existential) => {
                return (learned_clause, backtrack_level, result);
            },
            (Result::Restart, _) => {
                /*
                ---- Restart Handling ----
                Backtrack to level 1 to start from the beginning.
                Decide which learned conflicts to keep.
                */
                if matrix.decision_level != 1 {
                    return (learned_clause, backtrack_level, result);
                }
                matrix.reduce_clause_database();
                matrix.core_data.quantifier_list = pre_selection_quantifier_list;
                matrix.decision_level -= 1;
                continue;
            },
            (Result::Timeout, _) => {
                return (learned_clause, backtrack_level, result);
            }
        }
    }
}

/*
A function to cache the data structures that will need to be restored upon backtracking. 

Returns a cache of the current {clause database, clause references, quantifier prefix, trail, assignments, decision level).
*/
pub fn cache_necessary_structures(matrix: &CDCLMatrix) -> (ClauseSet, MultiMap<i32, i32>, Vec<Quantifier>, Vec<Assignment>, HashMap<i32, Assignment>, i32) {
    let current_clause_set = matrix.core_data.clause_set.clone();
    let current_clause_references = matrix.core_data.clause_references.clone();
    let current_quantifier_list = matrix.core_data.quantifier_list.clone();
    let current_trail = matrix.trail.clone();
    let current_assignments = matrix.assignments.clone();
    let current_decision_level = matrix.decision_level;
    return (current_clause_set, current_clause_references, current_quantifier_list, current_trail, current_assignments, current_decision_level);
}

/*
A function to restore the matrix with cached data structures during back-jumping/backtracking. 

Modifies the matrix and re-adds learned clauses so they're not lost upon back-jumping/backtracking.
*/
pub fn restore_necessary_structures(matrix: &mut CDCLMatrix, cached_structures: (ClauseSet, MultiMap<i32, i32>, Vec<Quantifier>, Vec<Assignment>, HashMap<i32, Assignment>, i32)) {
    matrix.core_data.clause_set = cached_structures.0;
    matrix.core_data.clause_references = cached_structures.1;
    matrix.core_data.quantifier_list = cached_structures.2;
    matrix.trail = cached_structures.3;
    matrix.assignments = cached_structures.4;
    matrix.decision_level = cached_structures.5;
    matrix.readd_learned_clauses();
}

/*
A function that defines the invariant to be returned within the cdcl procedure that signifies a satisfiable assignment.
*/
pub fn satisfiable() -> (Clause, i32, Result) {
    return (Clause::new_empty_clause(), -1, Result::SAT);
}

/*
A function that defines the invariant to be returned within the cdcl procedure that signifies an unsatisfiable assignment.
*/
pub fn unsatisfiable() -> (Clause, i32, Result) {
    return (Clause::new_empty_clause(), -1, Result::UNSAT);
}

/*
A function that defines the invariant to be returned within the cdcl procedure that signifies a timeout.
*/
pub fn timeout() -> (Clause, i32, Result) {
    return (Clause::new_empty_clause(), -1, Result::Timeout);
}

/*
A function to perform a restart on the matrix and update necessary data structures.

Returns an invariant to be returned within the cdcl procedure that signifies it should handle a Restart.
*/
pub fn perform_restart(matrix: &mut CDCLMatrix) -> (Clause, i32, Result) {
    matrix.restart_data.increment_restart_counter();
    matrix.restart_data.update_conflicts_until_restart(matrix.restart_data.restart_counter);
    matrix.restart_data.reset_current_conflicts();
    matrix.reset_conflict_clause();
    return (Clause::new_empty_clause(), -1, Result::Restart);
}