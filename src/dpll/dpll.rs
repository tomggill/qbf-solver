use std::time::Instant;

use crate::{dpll::unit_propagate::unit_propagate, data_structures::{Matrix, QuantifierType, Statistics, LiteralSelection}, literal_selection::{select_literal_vss, select_literal}};

/*
A struct to store the result of the DPLL procedure.

SAT => Satisfiable at current decision branch.
UNSAT => Unsatisfiable at current decision branch.
Timeout => Instance timeout, stop running the current instance.
*/
#[derive(Clone, Debug, PartialEq)]
pub enum Result {
    SAT,
    UNSAT,
    Timeout,
}

/*
A function that will perform the David-Putnam-LogemannLoveland (DPLL) algorithm with a selection of optimisations
from the set {Pure Literal Deletion, Universal Reduction, Pre-Resolution (done prior), Pre-Process (done prior)}.
Has one of the literal selection schemes {Ordered, Variable State Sum}.

Returns SAT (satisfiable), UNSAT (unsatisfiable), or Timeout.
*/
pub fn dpll(matrix: &mut Matrix, decision_branch: Option<i32>, statistics: &mut Statistics, timer: Instant) -> Result {
    if timer.elapsed().as_secs() > 30 { return Result::Timeout }

    let new_matrix = &mut matrix.clone();
    if !decision_branch.is_none() {
        unit_propagate(new_matrix, vec![decision_branch.unwrap()], statistics);
    }
    if new_matrix.clause_set.contains_empty_set() {
        return Result::SAT;
    } else if new_matrix.clause_set.contains_empty_clause() {
        return Result::UNSAT;
    }

    let (literal, quantifier_type) = if new_matrix.config.literal_selection.eq(&LiteralSelection::Ordered) 
                                                        {select_literal(new_matrix)} else {select_literal_vss(new_matrix)};

    let result = dpll(new_matrix, Some(literal), statistics, timer);
    match (&result, quantifier_type) {
        (Result::UNSAT, QuantifierType::Universal) => {
            return result;
        },
        (Result::SAT, QuantifierType::Universal) | (Result::UNSAT, QuantifierType::Existential) => {
            statistics.increment_backtrack_count();
            return dpll(new_matrix, Some(-literal), statistics, timer);
        },
        (Result::SAT, QuantifierType::Existential) => {
            return result;
        },
        (Result::Timeout, _) => {
            return result;
        }
    }
}