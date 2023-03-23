use multimap::MultiMap;
use crate::{data_structures::{Matrix, QuantifierType}, universal_reduction::{remove_universal_literal, detect_universal_literal}, util::get_quantifier_type};

/*
A function to get a list of pure literals from a given state.

Returns the list of pure literals.
*/
pub fn get_pure_literals(clause_references: &MultiMap<i32, i32>) -> Vec<i32> {
    let mut pure_literals = Vec::new();
    for key in clause_references.keys() {
        let complement_key = -key;
        if !clause_references.contains_key(&complement_key) {
            pure_literals.push(*key);
        }
    }
    return pure_literals;
}

/*
A function to will remove all pure literals from a given clause database, updating clause references where necessary.

Returns a list of unit literals detected during pure literal removal.
*/
pub fn remove_pure_literals(matrix: &mut Matrix, pure_literals: Vec<i32>) -> Vec<i32> {
    let mut new_unit_literals = Vec::new();
    for literal in pure_literals {
        let (quantifier_type, quantifier_position) = get_quantifier_type(&matrix.quantifier_list, literal);
        if !quantifier_position.is_none() {
            matrix.quantifier_list.remove(quantifier_position.unwrap());
        }
        let clause_references = matrix.clause_references.get_vec(&literal);
        if !clause_references.is_none() {
            for clause_index in clause_references.unwrap().clone() {
                if quantifier_type.eq(&QuantifierType::Existential) {
                    matrix.clause_set.clause_list[clause_index as usize].is_removed = true;
                    matrix.clause_set.decrement_counter();
                    matrix.clause_references.retain(|&_key, &value| { value != clause_index});
                    // Check satisfiability
                    if matrix.clause_set.contains_empty_set() {
                        return new_unit_literals;
                    }
                } else {
                    matrix.clause_set.clause_list[clause_index as usize].remove_a_literal(literal); // Only remove from a_literals as I know it is universally quantified.
                    matrix.clause_references.remove(&literal);

                    // Detect literals for Universal Reduction and remove them
                    if matrix.config.universal_reduction_enabled() {
                        let universal_literals = detect_universal_literal(&matrix.clause_set.clause_list[clause_index as usize], &matrix.variable_quantification);
                        if !universal_literals.is_empty() {
                            remove_universal_literal(matrix, universal_literals, clause_index);
                        }
                    }

                    // Check for contradiction
                    if matrix.clause_set.check_contradiction(Some(clause_index)) {
                        return new_unit_literals;
                    }

                    // Detect unit literals
                    let unit_clause_check = matrix.clause_set.clause_list[clause_index as usize].is_unit_clause();
                    if !unit_clause_check.is_none() {
                        new_unit_literals.push(unit_clause_check.unwrap());
                    }

                }
            }
        }
    }
    return new_unit_literals;
}