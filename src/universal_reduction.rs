use multimap::MultiMap;

use crate::{data_structures::{Clause, Variable, Matrix, UniversalReductionClause}, util::sort_literals_order};

/*
A function to get all universal literals that can be removed by universal reduction.

I have already sorted the literals stored in each clause in the order of their quantification from low-high. I only need to compare the outer u_literals to the highest quantification
e_literal so run-time reduced as I can break / continue depending on whether it was higher quantificaiton or not.

Returns a list of UniversalReductionClause data structures.
*/
pub fn get_universal_literals_for_reduction(clause_list: &Vec<Clause>, variable_quantification: &MultiMap<i32, Variable>) -> Vec<UniversalReductionClause> {
    let mut universal_literals = Vec::new();
    for (position, clause) in clause_list.iter().enumerate().rev() {
        let literals_to_remove = detect_universal_literal(clause, variable_quantification);
        if !literals_to_remove.is_empty() {
            universal_literals.push(UniversalReductionClause {
                clause_index: position as i32,
                values: literals_to_remove,
            });
        }
    }
    return universal_literals;
}

/*
A function to remove universal literals from a given clause.
*/
pub fn remove_universal_literal(matrix: &mut Matrix, literals: Vec<i32>, clause_index: i32) {
    matrix.clause_set.clause_list[clause_index as usize].remove_a_literals(literals);
    matrix.clause_set.check_contradiction(Some(clause_index));
}

/*
A function to restore clauses that had literals that were removed by universal reduction - necessary in CDCL.
*/
pub fn readd_universal_literal(matrix: &mut Matrix, literals: Vec<i32>, clause_index: i32) {
    let mut a_literals = Vec::from_iter(matrix.clause_set.clause_list[clause_index as usize].a_literals.clone());
    a_literals.extend(literals);
    let ordered_a_literals = sort_literals_order(&matrix.quantification_order.universal_literal_order, a_literals);
    matrix.clause_set.clause_list[clause_index as usize].replace_a_literals(ordered_a_literals);
}

/*
A function to detect any universal literals in a given clause which can be removed by universal reduction.

Returns the list of detected literals.
*/
pub fn detect_universal_literal(clause: &Clause, variable_quantification: &MultiMap<i32, Variable>) -> Vec<i32> {
    let mut literals_to_remove = Vec::new();
    for a_literal in clause.a_literals.iter().rev() {
        if clause.e_literals.is_empty() {
            literals_to_remove.extend(clause.a_literals.clone());
            break;
        }
        let max_quantification_e_literal = clause.e_literals[clause.e_literals.len() - 1];
        let a_literal_quantification = variable_quantification.get(&a_literal.abs());
        let e_literal_quantification = variable_quantification.get(&max_quantification_e_literal.abs());
        if !a_literal_quantification.is_none() && !e_literal_quantification.is_none() {
            if a_literal_quantification.unwrap().q_level > e_literal_quantification.unwrap().q_level {
                literals_to_remove.push(*a_literal);
            } else {
                break;
            }
        }
    }
    return literals_to_remove;
}