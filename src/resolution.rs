use std::collections::HashSet;
use multimap::MultiMap;

use crate::{data_structures::{Matrix, Clause, QuantifierType}, util::convert_literals_to_clause};

/*
A function to perform iterative pre-resolution on the clause database, adding resolved clauses to the original clause
database according to the pre-resolution hyperparameter configuration.

Note: original_clause_list is passed in when the solver type is CDCL.
*/
pub fn pre_resolution(matrix: &mut Matrix, original_clause_list: &mut Vec<Clause>) {
    let mut clause_hashtable = HashSet::new();
    clause_hashtable.extend(matrix.clause_set.clause_list.clone());
    let resolution_config = matrix.config.pre_resolution.1.clone();

    let clause_list = &mut matrix.clause_set.clause_list.clone();
    let clause_references = &mut matrix.clause_references.clone();
    let mut resolved_clause_database = Vec::new();

    let resolved_clauses_cap = (matrix.clause_set.clause_list.len() as f32 * resolution_config.max_ratio) as usize;
    let resolutions_per_literal = (matrix.clause_set.clause_list.len() as f32 * resolution_config.min_ratio) as usize / matrix.quantifier_list.len();
    for iteration in 0..resolution_config.iterations {
        let mut resolved_clauses = Vec::new();
        for quantifier in &matrix.quantifier_list {
            let mut resolved_clauses_for_literal = 0;
            if quantifier.q_type.eq(&QuantifierType::Existential) {
                let literal = quantifier.literal;
                if clause_references.contains_key(&literal) && clause_references.contains_key(&-literal) {
                    let pos_references = clause_references.get_vec(&literal).unwrap();
                    let neg_references = clause_references.get_vec(&-literal).unwrap();
                    for p_ref in pos_references {
                        let clause_1 = &clause_list[*p_ref as usize];
                        for n_ref in neg_references {
                            let clause_2 = &clause_list[*n_ref as usize];
                            let resolution = resolve(clause_1.clone().get_literal_list(), clause_2.clone().get_literal_list(), literal);
                            if resolution.is_none() {
                                continue;
                            } else {
                                let resolved_literals = resolution.unwrap();
                                let resolved_clause = convert_literals_to_clause(&matrix.variable_quantification, &matrix.quantification_order, &resolved_literals);
                                if !clause_hashtable.contains(&resolved_clause) {
                                    clause_hashtable.insert(resolved_clause.clone());
                                    resolved_clauses.push(resolved_clause);
                                    resolved_clauses_for_literal += 1;
                                    if resolved_literals.len() > resolution_config.repeat_above {
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            if resolved_clauses_for_literal >= resolutions_per_literal { break; }
                        }
                        if resolved_clauses_for_literal >= resolutions_per_literal { break; }
                    }
                }
            }
            if resolved_clauses.len() > resolved_clauses_cap { break; }
        }

        // No need to continue resolution if we didnt produce any new resolved clauses
        if resolved_clauses.is_empty() { break };
        resolved_clause_database.extend(resolved_clauses.clone());
        if iteration < resolution_config.iterations - 1 { // i.e it is not the last iteration.
            add_resolved_clauses_independently(clause_list, clause_references, resolved_clauses);
        }
    }
    add_resolved_clauses(matrix, resolved_clause_database, resolution_config.max_clause_length, original_clause_list);
}

/*
A function to perform Q-Resolution on a literal for two given clause literal lists given it's existentially 
quantified (I am not dealing with cubes). If for any variable, the resolved clause also contains its complement, 
the resolution is unsound and invalid. In this case I return None.
*/
pub fn resolve(literals_list_1: Vec<i32>, literals_list_2: Vec<i32>, literal: i32) -> Option<Vec<i32>> {
    let mut resolved_literals: HashSet<i32> = HashSet::from_iter(literals_list_1.clone());
    resolved_literals.extend(literals_list_2);
    resolved_literals.remove(&literal);
    resolved_literals.remove(&-literal);
    let mut literals_checked = HashSet::new();
    let mut invalid = false;
    for x in resolved_literals.iter() {
        if literals_checked.contains(&-x) {
            invalid = true;
            break;
        } else {
            literals_checked.insert(*x);
        }
    }
    return if invalid { None } else { Some(Vec::from_iter(resolved_literals)) };
}

/*
A function to add a list of clauses to the clause database of a given ParsedMatrix/Problem. It will update necessary 
variable states such as clause references.
*/
pub fn add_resolved_clauses(matrix: &mut Matrix, resolved_clauses: Vec<Clause>, max_clause_length: usize, original_clause_list: &mut Vec<Clause>) {
    let mut clause_index = matrix.clause_set.clause_list.len() as i32 - 1;
    for clause in resolved_clauses {
        if clause.get_clause_length() > max_clause_length { continue }
        matrix.clause_set.clause_list.push(clause.clone());
        matrix.clause_set.clause_count += 1;
        if !original_clause_list.is_empty() {
            original_clause_list.push(clause.clone());
        }
        clause_index += 1;
        for literal in clause.get_literal_list() {
            matrix.clause_references.insert(literal, clause_index as i32);
        }
    }
}

/*
A function to add a list of resolved clauses to the main clause list, updating the references for the main clause list.
This is done independently of the matrix structure which is necessary for iterative pre-resolution.
*/
pub fn add_resolved_clauses_independently(clause_list: &mut Vec<Clause>, clause_references: &mut MultiMap<i32, i32>, resolved_clauses: Vec<Clause>) {
    let mut clause_index = clause_list.len() as i32 - 1;
    for clause in resolved_clauses {
        clause_list.push(clause.clone());
        clause_index += 1;
        for literal in clause.get_literal_list() {
            clause_references.insert(literal, clause_index as i32);
        }
    }
}