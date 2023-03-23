use std::collections::VecDeque;
use crate::{data_structures::{Matrix, QuantifierType, Statistics}, universal_reduction::{remove_universal_literal, detect_universal_literal}, util::get_quantifier_type, pure_literal_deletion::{remove_pure_literals, get_pure_literals}};

/*
A function to perform unit propagation (Boolean Constraint Propagation) on a given Matrix data structure.

It will subsequently perform pure literal deletion, universal reduction, and further unit propagation when possible.
It will check for the empty set of clauses and the empty clause and return flags for handling satisfiable and 
unsatisfiable assignments.
*/
pub fn unit_propagate(matrix: &mut Matrix, unit_literal: Vec<i32>, statistics: &mut Statistics) {
    let mut new_unit_literals = VecDeque::new();
    new_unit_literals.extend(unit_literal);
    while !new_unit_literals.is_empty() {
        statistics.increment_propagation_count();
        let temp_unit_literal: i32 = new_unit_literals.pop_front().unwrap();
        let (quantifier_type, quantifier_position) = get_quantifier_type(&matrix.quantifier_list, temp_unit_literal);
        // If the literal we are propagating is quantified, remove it from the quantifier prefix.
        if !quantifier_position.is_none() {
            matrix.quantifier_list.remove(quantifier_position.unwrap());
        }
        if quantifier_type.eq(&QuantifierType::Universal) {
            matrix.clause_set.clause_count = -1;
            return;
        } else {
            let pos_clause_references = matrix.clause_references.get_vec(&temp_unit_literal);
            if !pos_clause_references.is_none() {
                for clause_index in pos_clause_references.unwrap().clone() {
                    matrix.clause_set.clause_list[clause_index as usize].is_removed = true; // Mark clause as removed
                    matrix.clause_set.decrement_counter();
                    matrix.clause_references.retain(|&_key, &value| { value != clause_index});
                    // Check satisfiability
                    if matrix.clause_set.contains_empty_set() {
                        return;
                    }
                }
            }
            let complement_unit_literal = -temp_unit_literal;
            let neg_clause_references = matrix.clause_references.get_vec(&complement_unit_literal);
            if !neg_clause_references.is_none() {
                let definitive_q_type = &matrix.variable_quantification.get(&temp_unit_literal.abs()).unwrap().q_type.clone();
                for clause_index in neg_clause_references.unwrap().clone()  {
                    if definitive_q_type.eq(&QuantifierType::Existential) {
                        matrix.clause_set.clause_list[clause_index as usize].remove_e_literal(complement_unit_literal);
                    } else {
                        matrix.clause_set.clause_list[clause_index as usize].remove_a_literal(complement_unit_literal);
                    }
                    matrix.clause_references.remove(&complement_unit_literal); // Remove map index for the complement unit literal as it has been removed.
                    // Check for contradiction
                    if matrix.clause_set.check_contradiction(Some(clause_index)) {
                        return;
                    }

                    if matrix.config.universal_reduction_enabled() {
                        let universal_literals = detect_universal_literal(&matrix.clause_set.clause_list[clause_index as usize], &matrix.variable_quantification);
                        if !universal_literals.is_empty() {
                            remove_universal_literal(matrix, universal_literals, clause_index);
                            if matrix.clause_set.check_contradiction(None) {
                                return;
                            }
                        }
                    }

                    // Check for new unit clauses
                    let unit_clause_check = matrix.clause_set.clause_list[clause_index as usize].is_unit_clause();
                    if !unit_clause_check.is_none() {
                        new_unit_literals.push_back(unit_clause_check.unwrap());
                    }
                }
            }
        }
        // Pure literals can cause the detection of literals for universal reduction.
        if matrix.config.pure_literal_deletion_enabled() && new_unit_literals.is_empty() {
            let pure_literals = get_pure_literals(&matrix.clause_references);
            if !pure_literals.is_empty() {
                let detected_unit_literals = remove_pure_literals(matrix, pure_literals);
                if matrix.clause_set.check_contradiction(None) {
                    return;
                }
                new_unit_literals.extend(detected_unit_literals);
            }
        }
    }
    return;
}