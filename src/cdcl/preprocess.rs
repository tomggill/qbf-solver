use std::time::Instant;

use multimap::MultiMap;

use crate::{cdcl::unit_propagate::unit_propagate, data_structures::{CDCLMatrix, Statistics}, universal_reduction::{get_universal_literals_for_reduction, remove_universal_literal}, pure_literal_deletion::{remove_pure_literals, get_pure_literals}, util::get_unit_literals};

/*
A function to reduce the initial problem set by applying pre-processing techniques unit propagation, universal reduction,
and pure literal removal iteratively until no longer possible.
*/
pub fn preprocess(matrix: &mut CDCLMatrix, statistics: &mut Statistics, timer: Instant) {
    let mut is_finished = false;
    let mut pure_literals;
    let mut literals_for_universal_reduction;
    let mut unit_literals;
    while !is_finished {
        // Timeout the instance after 30 seconds 
        if timer.elapsed().as_secs() > 30 { return; };

        // Perform unit propagation on the set of clauses
        unit_literals = get_unit_literals(&matrix.core_data.clause_set.clause_list);
        if !unit_literals.is_empty() {
            unit_propagate(matrix, unit_literals, false, statistics);
        }
        if matrix.core_data.check_solved() { break; }

        // Perform pure literal deletion on the set of clauses
        if matrix.core_data.config.pure_literal_deletion_enabled() {
            pure_literals = get_pure_literals(&matrix.core_data.clause_references);
            if !pure_literals.is_empty() {
                remove_pure_literals(&mut matrix.core_data, pure_literals);
            }
            if matrix.core_data.check_solved() { break; }
        }

        // Perform universal reduction on the set of clauses
        if matrix.core_data.config.universal_reduction_enabled() {
            literals_for_universal_reduction = get_universal_literals_for_reduction(&matrix.core_data.clause_set.clause_list, &matrix.core_data.variable_quantification);
            if !literals_for_universal_reduction.is_empty() {
                for literal_to_remove in literals_for_universal_reduction {
                    remove_universal_literal(&mut matrix.core_data, literal_to_remove.values, literal_to_remove.clause_index);
                }
            }
            if matrix.core_data.check_solved() { break; }
        }
        pure_literals = if matrix.core_data.config.pure_literal_deletion_enabled() {get_pure_literals(&matrix.core_data.clause_references) } else { Vec::new() };
        literals_for_universal_reduction = if matrix.core_data.config.universal_reduction_enabled() { get_universal_literals_for_reduction(&matrix.core_data.clause_set.clause_list, &matrix.core_data.variable_quantification) } else { Vec::new() };
        unit_literals = get_unit_literals(&matrix.core_data.clause_set.clause_list);
        if pure_literals.is_empty() && literals_for_universal_reduction.is_empty() && unit_literals.is_empty() {
            is_finished = true;
        }
    }
    simplify_constraints(matrix);
}

/*
Function to simplify the problem set constraints. It will permanently remove any clauses that are no longer impacting 
the problem, and it will update the clause references where appropriate.
*/
pub fn simplify_constraints(matrix: &mut CDCLMatrix) {
    let mut remove_clause_references = Vec::new();
    for (index, clause) in matrix.core_data.clause_set.clause_list.iter().enumerate() {
        if clause.is_removed {
            remove_clause_references.push(index as usize);
        }
    }
    for reference in remove_clause_references.iter().rev() {
        matrix.core_data.clause_set.clause_list.remove(*reference);
        matrix.learned_clause_refs.retain(|&x| x != *reference as i32);
        for (index, learned_clause_reference) in matrix.learned_clause_refs.clone().iter().enumerate() {
            if learned_clause_reference > &(*reference as i32) {
                matrix.learned_clause_refs[index] -= 1;
            }
        }
    }
    let mut clause_references = MultiMap::new();
    for (index, clause) in matrix.core_data.clause_set.clause_list.iter().enumerate() {
        for literal in clause.clone().get_literal_list() {
            clause_references.insert(literal, index as i32);
        }
    }
    matrix.restart_data.current_conflicts = 0; // Since we are refreshing the database, set current conflicts to 0.
    matrix.core_data.clause_references = clause_references;
    matrix.original_clause_list = matrix.core_data.clause_set.clause_list.clone();
}