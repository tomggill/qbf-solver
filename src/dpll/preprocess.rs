use std::time::Instant;

use multimap::MultiMap;

use crate::{dpll::unit_propagate::unit_propagate, data_structures::{Matrix, Statistics}, universal_reduction::{remove_universal_literal, get_universal_literals_for_reduction}, pure_literal_deletion::{get_pure_literals, remove_pure_literals}, util::get_unit_literals};

/*
A function to reduce the initial problem set by applying pre-processing techniques unit propagation, universal reduction,
and pure literal removal iteratively until no longer possible.
*/
pub fn preprocess(matrix: &mut Matrix, statistics: &mut Statistics, timer: Instant) {
    let mut is_finished = false;
    let mut pure_literals;
    let mut literals_for_universal_reduction;
    let mut unit_literals;
    while !is_finished {
        // Timeout the instance after 30 seconds 
        if timer.elapsed().as_secs() > 30 { return; };

        // Perform unit propagation on the set of clauses
        unit_literals = get_unit_literals(&matrix.clause_set.clause_list);
        if !unit_literals.is_empty() {
            unit_propagate(matrix, unit_literals, statistics);
        }
        if matrix.check_solved() { break; }

        // Perform pure literal deletion on the set of clauses
        if matrix.config.pure_literal_deletion_enabled() {
            pure_literals = get_pure_literals(&matrix.clause_references);
            if !pure_literals.is_empty() {
                remove_pure_literals(matrix, pure_literals);
            }
            if matrix.check_solved() { break; }
        }

        // Perform universal reduction on the set of clauses
        if matrix.config.universal_reduction_enabled() {
            literals_for_universal_reduction = get_universal_literals_for_reduction(&matrix.clause_set.clause_list, &matrix.variable_quantification);
            if !literals_for_universal_reduction.is_empty() {
                for literal_to_remove in literals_for_universal_reduction {
                    remove_universal_literal(matrix, literal_to_remove.values, literal_to_remove.clause_index);
                }
            }
            if matrix.check_solved() { break; }
        }
        pure_literals = if matrix.config.pure_literal_deletion_enabled() {get_pure_literals(&matrix.clause_references) } else { Vec::new() };
        literals_for_universal_reduction = if matrix.config.universal_reduction_enabled() { get_universal_literals_for_reduction(&matrix.clause_set.clause_list, &matrix.variable_quantification) } else { Vec::new() };
        unit_literals = get_unit_literals(&matrix.clause_set.clause_list);
        if pure_literals.is_empty() && literals_for_universal_reduction.is_empty() && unit_literals.is_empty() {
            is_finished = true;
        }
    }
    simplify_constraints(matrix);
}

/*
A function to simplify the problem set constraints. It will permanently remove any clauses that are no longer impacting 
the problem, and it will update the clause references where appropriate.
*/
pub fn simplify_constraints(matrix: &mut Matrix) {
    let mut remove_clause_references = Vec::new();
    for (index, clause) in matrix.clause_set.clause_list.iter().enumerate() {
        if clause.is_removed {
            remove_clause_references.push(index as usize);
        }
    }
    for reference in remove_clause_references.iter().rev() {
        matrix.clause_set.clause_list.remove(*reference);
    }
    let mut clause_references = MultiMap::new();
    for (index, clause) in matrix.clause_set.clause_list.iter().enumerate() {
        for literal in clause.clone().get_literal_list() {
            clause_references.insert(literal, index as i32);
        }
    }
    matrix.clause_references = clause_references;
}