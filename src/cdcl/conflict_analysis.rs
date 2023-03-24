use std::cmp;
use crate::{data_structures::{CDCLMatrix, QuantifierType, Clause, Statistics}, util::convert_literals_to_clause, resolution::resolve};

/*
A function to get the literal with the highest decision level from a list of literals.

Returns (the highest decision literals, the highest decision level)
*/
pub fn get_highest_decision_level(matrix: &CDCLMatrix, literals: &Vec<i32>) -> (i32, i32) {
    let mut highest_decision_level = -1;
    let mut highest_decision_literal = -1;
    for literal in literals {
        let quantification_type = &matrix.core_data.variable_quantification.get(&literal.abs()).expect("Variable quantification missing literal").q_type;
        let assignment = matrix.assignments.get(&literal.abs()).expect("Assignment store missing literal");
        if quantification_type.eq(&QuantifierType::Existential) {
            if highest_decision_level < assignment.decision_level {
                highest_decision_level = assignment.decision_level;
                highest_decision_literal = *literal;
            }
        }
    }
    return (highest_decision_literal, highest_decision_level);
}

/*
--- Stopping Constraint 1 ---
This function checks the first stopping constraint for my conflict analysis procedure. It checks that amongst all
the existential literals in the resolved clause, only one of them is at the highest decision level.

Returns (the highest decision literal, the highest decision level, whether constraint is met or not).
*/
pub fn contains_one_highest_decision_literal(matrix: &CDCLMatrix, literals: &Vec<i32>) -> (i32, i32, bool) {
    let (v, highest_decision_level) = get_highest_decision_level(matrix, literals);
    let mut two_highest_decision_literals = false;
    for literal in literals {
        let quantification_type = &matrix.core_data.variable_quantification.get(&literal.abs()).expect("Variable quantification missing literal").q_type;
        let assignment = matrix.assignments.get(&literal.abs()).expect("Assignment store missing literal");
        if quantification_type.eq(&QuantifierType::Existential) {
            if assignment.decision_level == highest_decision_level && v != *literal {
                two_highest_decision_literals = true;
                break;
            }
        }
    }
    return (v, highest_decision_level, !two_highest_decision_literals);
}

/*
--- Stopping Constraint 2 ---
This function checks the second stopping constraint for my conflict analysis procedure. It checks that the highest
decision literal is at a decision level with an existential variable as its branch variable (decision).

Returns (whether the constraint is met or not).
*/
pub fn contains_highest_decision_level_decision(matrix: &CDCLMatrix, highest_decision_level: i32) -> bool {
    let mut new_trail = matrix.trail.clone();
    let mut is_existential = false;
    loop {
        let assignment = new_trail.pop().expect("Trail missing assignment literal");
        if assignment.decision_level == highest_decision_level {
            if assignment.is_decision() { 
                let quantification = &matrix.core_data.variable_quantification.get(&assignment.value.abs()).expect("Variable quantification missing literal").q_type;
                if quantification.eq(&QuantifierType::Existential) {
                    is_existential = true;
                }
                break;
            }
        }

        if assignment.decision_level < highest_decision_level { break };
    }
    return is_existential;
}

/*
--- Stopping Constraint 3 ---
This function checks the third stopping constraint for my conflict analysis procedure. It checks that all universally
quantified literals with a smaller quantification level than the highest decision literal are assigned 0.

Returns (whether the constraint is met or not).
*/
pub fn all_previous_universals_assigned_correctly(matrix: &CDCLMatrix, literals: &Vec<i32>, highest_decision_literal: i32) -> bool {
    let mut is_valid = true;
    let hdl_quantification_level = matrix.core_data.variable_quantification.get(&highest_decision_literal.abs()).expect("Variable quantification missing literal").q_level;
    for literal in literals {
        let quantification_variable = matrix.core_data.variable_quantification.get(&literal.abs()).expect("Variable quantification missing literal");
        if quantification_variable.q_type.eq(&QuantifierType::Universal) {
            if quantification_variable.q_level < hdl_quantification_level {
                let assignment = matrix.assignments.get(&literal.abs()).expect("Assignment store missing literal");
                if assignment.value != -literal {
                    is_valid = false;
                    break;
                }
            }
        }
    }
    return is_valid;
}

/*
Upon meeting the above constraints, this function calculates which level to backtrack to based upon the first unique
implication point. It will backtrack to a point where a literal in the newly learned clause will be made unit, which
will fuel further implications.

Returns (the backtrack level).
*/
pub fn calculate_backtrack_level(matrix: &CDCLMatrix, literals: &Vec<i32>, highest_decision_level: i32) -> i32 {
    let mut backtrack_level = -1;
    for literal in literals {
        let assignment = matrix.assignments.get(&literal.abs()).expect("Assignment store missing literal");
        if assignment.decision_level == highest_decision_level {
            continue;
        }
        backtrack_level = cmp::max(backtrack_level, assignment.decision_level);
    }
    // Catch edge cases.
    if backtrack_level == -1 { backtrack_level = highest_decision_level - 1 }
    if literals.len() > 1 && backtrack_level == 0 { backtrack_level = 1 }
    return backtrack_level
}

/*
Checks whether the learned clause results in unsatisfiability. This is the case if either:
- All existential literals in the learned clause are at decision level 0;
- All literals in the clause are universal.

Returns (whether it is unsatisfiable).
*/
pub fn check_unsatisfiability_criteria(matrix: &CDCLMatrix, literals: &Vec<i32>) -> bool {
    let mut only_universals = true;
    let mut existentials_at_level_0 = true;
    for literal in literals {
        let quantification = &matrix.core_data.variable_quantification.get(&literal.abs()).expect("Variable quantification missing literal").q_type;
        if quantification.eq(&QuantifierType::Existential) {
            let assignment = matrix.assignments.get(&literal.abs()).expect("Assignment store missing literal");
            if assignment.decision_level > 0 {
                existentials_at_level_0 = false;
            }
            only_universals = false;
        }
    }
    if !only_universals && !existentials_at_level_0 {
        return false;
    } else {
        return true;
    }
}

/*
This function will analyse a given conflict given it occurs on an existential literal assignment. It will iteratively
perform Q-Resolution on the conflict clause and its literals until certain stopping constraints are met. These ensure 
that upon backjumping/backtracking the learned clause is unit, based upon 1UIP, and that we don't only undo universal
decisions as they can't resolve a conflict.

Note: On unsatisfiability, it will return the empty clause and a backtrack level of -1. This will exit the procedure 
and return unsatisfiable.

Returns (the learned clause, backtrack_level)
*/
pub fn analyse_conflict(matrix: &mut CDCLMatrix, statistics: &mut Statistics) -> (Clause, i32) {
    // If conflict hit as a direct result of a universal literal, conflict learning is not applicable so naively backtrack. 
    if matrix.conflict_clause.is_none() {
        return (Clause::new_empty_clause(), matrix.decision_level);
    }
    statistics.increment_learned_clause_count();
    let conflict = matrix.conflict_clause.clone().expect("Conflict clause expected in analyse_conflict");
    matrix.reset_conflict_clause();
    let mut trail = matrix.trail.clone();
    let mut current_literals = conflict.get_literal_list();
    let mut backtrack_level;
    loop {
        if trail.len() == 0 {
            let (_highest_decision_literal, highest_decision_level, _constraint_one) = contains_one_highest_decision_literal(matrix, &current_literals);
            backtrack_level = calculate_backtrack_level(matrix, &current_literals, highest_decision_level);
            break;
        }
        let mut resolution_occurred = false;
        let assignment = trail.pop().unwrap();
        if !assignment.is_decision() {
            let quantification_type = &matrix.core_data.variable_quantification.get(&assignment.value.abs()).unwrap().q_type;
            if quantification_type.eq(&QuantifierType::Existential) {
                if current_literals.contains(&assignment.value) || current_literals.contains(&-assignment.value) {
                    let clause_responsible = matrix.original_clause_list[assignment.clause_responsible.unwrap() as usize].clone();
                    let resolved_literals = resolve(current_literals, clause_responsible.get_literal_list(), assignment.value).expect("Resolution shouldn't be invalid here.");
                    current_literals = resolved_literals;
                    // Check unsatisfiability constraints.
                    if check_unsatisfiability_criteria(matrix, &current_literals) {
                        return (Clause::new_empty_clause(), -1);
                    }
                    resolution_occurred = true;
                }
            }
        }
        if !resolution_occurred { continue }; // If no new resolution, constraints still not met.

        // Stopping constraint 1 - Among all its existential variables, only one of them has the highest decision level.
        let (highest_decision_literal, highest_decision_level, constraint_one) = contains_one_highest_decision_literal(matrix, &current_literals);
        if !constraint_one { continue };

        // Stopping constraint 2 - The highest decision literal is in a decision level with an existential variable as the decision variable.
        let constraint_two =  contains_highest_decision_level_decision(matrix, highest_decision_level);
        if !constraint_two { continue };

        // Stopping constraint 3 - All universal literals with quantification level smaller than the highest
        // decision literal are assigned 0 prior.
        let constraint_three = all_previous_universals_assigned_correctly(matrix, &current_literals, highest_decision_literal);
        if !constraint_three { continue };

        // Determine level to backtrack to.
        backtrack_level = calculate_backtrack_level(matrix, &current_literals, highest_decision_level);
        break;
    }
    // If learned clause is a unit clause, I want to backtrack to level 0 and simplify the problem.
    if current_literals.len() == 1 {
        backtrack_level = 0;
    }
    let clause = convert_literals_to_clause(&matrix.core_data.variable_quantification, &matrix.core_data.quantification_order, &current_literals);

    return (clause, backtrack_level); // if backtrack_level = -1 --> return unsatisfiable
}