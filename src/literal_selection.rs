use crate::{data_structures::{Matrix, QuantifierType}, util::get_variable_state_sum};

/*
A function to select a literal from the outermost quantifier from the quantification prefix. It will not select literals 
which don't appear in the set of clauses, removing them from the quantifier prefix.

Returns (the selected literal, quantification type of the literal).
*/
pub fn select_literal(matrix: &mut Matrix) -> (i32, QuantifierType) {
    let mut quantifier = matrix.quantifier_list.remove(0);
    let mut literal = quantifier.literal;
    while !matrix.clause_references.contains_key(&literal) && !matrix.clause_references.contains_key(&-literal) {
        quantifier = matrix.quantifier_list.remove(0);
        literal = quantifier.literal;
    }
    let quantifier_type = quantifier.q_type;
    return (literal, quantifier_type);
}

/*
A function to select a literal from the outer quantification set based on the literals variable state sum.
It will not select literals which don't appear in the set of clauses, removing them from the quantifier prefix.

Explanation: ∃123∀46∃5(C), I can select literals from the set {1, 2, 3} in any order. Only after propagating all
these literals can I select from the next quantification set ∀46.

Returns (the selected literal, quantification type of the literal).
*/
pub fn select_literal_vss(matrix: &mut Matrix) -> (i32, QuantifierType) {
    let mut max_appearences = 0;
    let mut remove_indices = Vec::new();
    let mut choice = 0;
    let mut top_level_quantification = &matrix.quantifier_list.get(0).unwrap().q_type;
    let mut choose_positive = true;
    for (index, q) in matrix.quantifier_list.iter().enumerate() {
        if !matrix.clause_references.contains_key(&q.literal) && !matrix.clause_references.contains_key(&-q.literal) {
            remove_indices.push(index);
            continue;
        }
        // Move to next quantifier type if necessary.
        if q.q_type.ne(top_level_quantification) {
            if max_appearences > 0 {
                break;
            } else { 
                top_level_quantification = &q.q_type;
            }
        }
        let (current_literal_appearances, priority) = get_variable_state_sum(&matrix.clause_references, q.literal);

        if current_literal_appearances > max_appearences {
            choose_positive = priority;
            max_appearences = current_literal_appearances;
            choice = index;
        }
    }
    let quantifier = matrix.quantifier_list.remove(choice);
    let literal = if choose_positive {quantifier.literal} else {-quantifier.literal};
    let quantifier_type = quantifier.q_type;
    // This loop ensures that the quantifier prefix structure is updated correctly.
    for index in remove_indices.iter().rev() {
        if index.gt(&choice) {
            matrix.quantifier_list.remove(*index - 1);
        } else {
            matrix.quantifier_list.remove(*index);
        }
    }
    return (literal, quantifier_type);
}