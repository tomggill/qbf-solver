use std::collections::{HashMap, VecDeque};

use crate::{data_structures::{CDCLMatrix, Assignment, QuantifierType, Statistics}, util::get_quantifier_type, universal_reduction::{detect_universal_literal, remove_universal_literal, readd_universal_literal}};

/*
A function to perform unit propagation (Boolean Constraint Propagation) on a given CDCLMatrix data structure.

It will subsequently perform universal reduction and further unit propagation when possible.
It will check for the empty set of clauses and the empty clause and return flags for handling satisfiable and 
unsatisfiable assignments.
*/
pub fn unit_propagate(matrix: &mut CDCLMatrix, unit_literal: Vec<i32>, decision: bool, statistics: &mut Statistics) {
    let mut new_unit_literals = VecDeque::new();
    let mut implied_clause_references = HashMap::new();
    new_unit_literals.extend(&unit_literal);
    while !new_unit_literals.is_empty() {
        statistics.increment_propagation_count();
        let temp_unit_literal = new_unit_literals.pop_front().unwrap();
        // Assign to trail and assignments.
        if decision {
            let clause_index = implied_clause_references.get(&temp_unit_literal).copied();
            let new_assignment = Assignment {
                value: temp_unit_literal,
                decision_level: matrix.decision_level,
                clause_responsible: clause_index,
            };
            matrix.trail.push(new_assignment.clone());
            matrix.assignments.insert(temp_unit_literal.abs(), new_assignment);
        }


        let (quantifier_type, quantifier_position) = get_quantifier_type(&matrix.core_data.quantifier_list, temp_unit_literal);
        // If the literal we are propagating is quantified, remove it from the quantifier prefix.
        if !quantifier_position.is_none() {
            matrix.core_data.quantifier_list.remove(quantifier_position.unwrap());
        }
        if quantifier_type.eq(&QuantifierType::Universal) {
            matrix.core_data.clause_set.clause_count = -1;
            return;
        } else {
            let pos_clause_references = matrix.core_data.clause_references.get_vec(&temp_unit_literal);
            if !pos_clause_references.is_none() {
                for clause_index in pos_clause_references.unwrap().clone() {
                    matrix.core_data.clause_set.clause_list[clause_index as usize].is_removed = true; // Mark clause as removed
                    matrix.core_data.clause_set.decrement_counter();
                    matrix.core_data.clause_references.retain(|&_key, &value| { value != clause_index});
                    // Check satisfiability
                    if matrix.core_data.clause_set.contains_empty_set() {
                        return;
                    }
                }
            }
            let complement_unit_literal = -temp_unit_literal;
            let neg_clause_references = matrix.core_data.clause_references.get_vec(&complement_unit_literal);
            if !neg_clause_references.is_none() {
                let definitive_q_type = &matrix.core_data.variable_quantification.get(&temp_unit_literal.abs()).unwrap().q_type.clone();
                for clause_index in neg_clause_references.unwrap().clone()  {
                    if definitive_q_type.eq(&QuantifierType::Existential) {
                        matrix.core_data.clause_set.clause_list[clause_index as usize].remove_e_literal(complement_unit_literal);
                    } else {
                        matrix.core_data.clause_set.clause_list[clause_index as usize].remove_a_literal(complement_unit_literal);
                    }
                    matrix.core_data.clause_references.remove(&complement_unit_literal);

                    if matrix.core_data.config.universal_reduction_enabled() {
                        let universal_literals = detect_universal_literal(&matrix.core_data.clause_set.clause_list[clause_index as usize], &matrix.core_data.variable_quantification);
                        if !universal_literals.is_empty() {
                            remove_universal_literal(&mut matrix.core_data, universal_literals.clone(), clause_index);
                            if matrix.core_data.clause_set.check_contradiction(None) {
                                matrix.core_data.clause_set.clause_count = -1;
                                return;
                            } else {
                                readd_universal_literal(&mut matrix.core_data, universal_literals, clause_index);
                            }
                        }
                    }

                    // Check for contradiction
                    if matrix.core_data.clause_set.check_contradiction(Some(clause_index)) {
                        let conflict = matrix.original_clause_list[clause_index as usize].clone();
                        matrix.conflict_clause = Some(conflict);
                        return;
                    }

                    // Check for new unit clauses
                    let unit_clause_check = matrix.core_data.clause_set.clause_list[clause_index as usize].is_unit_clause();
                    if !unit_clause_check.is_none() {
                        let found_unit_clause = unit_clause_check.unwrap();
                        if !new_unit_literals.contains(&found_unit_clause) {
                            implied_clause_references.insert(found_unit_clause, clause_index);
                            new_unit_literals.push_back(found_unit_clause);
                        }
                    }
                }
            }
        }
    }
    return;
}