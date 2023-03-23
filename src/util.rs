use multimap::MultiMap;
use regex::Regex;

use crate::data_structures::{Clause, QuantifierType, Variable, QuantificationOrder, Quantifier};

/*
A function to sort a list of literals into the order in which the variables appear quantified.

Returns the sorted list of literals.
*/
pub fn sort_literals_order(sort_order: &Vec<i32>, literals: Vec<i32>) -> Vec<i32> {
    let mut sorted_literals = literals.clone();
    sorted_literals.sort_by(|a, b| {
        let index_a = sort_order.iter().position(|&r| r == *a || r == (-a));
        let index_b = sort_order.iter().position(|&r| r == *b || r == (-b));
        return index_a.cmp(&index_b);
    });
    return sorted_literals;
}

/*
A function to find the number of references a literal has in the current matrix. It also determines the sign priority.

choose_positive determines whether the variable should be decided positively or negatively. If the variable appears more 
often negatively, we choose the variable negatively. Otherwise, we choose the variable positively.
*/
pub fn get_variable_state_sum(clause_references: &MultiMap<i32, i32>, literal: i32) -> (i32, bool) {
    let mut pos_appearances = 0;
    if clause_references.contains_key(&literal) {
        pos_appearances += clause_references.get_vec(&literal).unwrap().len() as i32;
    }
    let mut neg_appearances = 0;
    if clause_references.contains_key(&-literal) {
        neg_appearances += clause_references.get_vec(&-literal).unwrap().len() as i32;
    }
    let choose_positive = if neg_appearances >= pos_appearances {false} else {true};
    let appearances = pos_appearances + neg_appearances;
    return (appearances, choose_positive);
}

/*
A function to convert a list of literals into clause structure, with sorted literals in their quantification ordering.

Returns the created clause.
*/
pub fn convert_literals_to_clause(variable_quantification: &MultiMap<i32, Variable>, quantification_order: &QuantificationOrder, literals: &Vec<i32>) -> Clause {
    let mut e_literals = Vec::new();
    let mut a_literals = Vec::new();
    for literal in literals {
        let literal_quantification = &variable_quantification.get(&literal.abs()).unwrap().q_type;
        if literal_quantification.eq(&QuantifierType::Existential) {
            e_literals.push(*literal);
        } else {
            a_literals.push(*literal);
        }
    }
    e_literals = sort_literals_order(&quantification_order.existential_literal_order, e_literals);
    a_literals = sort_literals_order(&quantification_order.universal_literal_order, a_literals);

    let resolved_clause = Clause {
        e_literals,
        a_literals,
        is_removed: false,
    };
    return resolved_clause;
}

/*
A function to get the quantifier type.

Note: It will return existential for a universally quantified literal if it's been decided prior as it doesn't appear
within the quantifier prefix. This is correct as I want to handle it the same way for both  quantifier types
when it is a decision literal.

Returns the quantification type and the index the quantifier appears in the quantifier prefix.
*/
pub fn get_quantifier_type(quantifier_list: &Vec<Quantifier>, unit_literal: i32) -> (QuantifierType, Option<usize>) {
    for (index, quantifier) in quantifier_list.iter().enumerate() {
        if quantifier.literal == unit_literal || quantifier.literal == -unit_literal {
            return (quantifier.q_type.clone(), Some(index));
        }
    }
    // This means it is not quantified - in thise case we treat both universal and and existential literals the 
    // same within unit propagation but handle backtracking differently.
    return (QuantifierType::Existential, None); 
}

/*
A function to check for unit literals in a list of clauses.

Returns a list of unit literals.
*/
pub fn get_unit_literals(clause_list: &Vec<Clause>) -> Vec<i32> {
    let mut unit_literals = Vec::new();
    for clause in clause_list {
        let unit_clause_check = clause.is_unit_clause();
        if !unit_clause_check.is_none() {
            unit_literals.push(unit_clause_check.unwrap());
        }
    }
    return unit_literals;
}

/*
A function to get the instance name from a file_path.

Example: file_path = ./benchmarks/castellini\toilet_a_02_10.2.qdimacs
            => instance_name = toilet_a_02_10.2.qdimacs

Returns the instance name.
*/
pub fn read_instance_name(file_path: &String) -> String {
    let re_get_instance = Regex::new(r"[^\\]+$").unwrap();
    let instance_name = re_get_instance.find(&file_path).map(|m| m.as_str()).unwrap().to_string();
    return instance_name;
}