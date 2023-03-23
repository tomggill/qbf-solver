use std::{fs::File, io::{self, BufRead}, path::Path, collections::HashMap};
use multimap::MultiMap;

use crate::util::sort_literals_order;


/*
An enum to store the type of solver algorithm to run.
*/
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SolverType {
    DPLL,
    CDCL,
}

/*
A struct to store:
- the solver type
- whether you are running a benchmark or instance,
- the benchmark directory path or the instance file path
- the filename you want the results stored in
*/
pub struct Solver {
    pub solver_type: SolverType,
    pub run_bench: bool,
    pub path: String,
    pub output: String,
}

/*
A struct to store the hyperparameters governing how pre-resolution is ran.

min_ratio: Min clause percentage of original clause database
max_ratio: Max clause percentage of original clause database
max_clause_length: Don't add resolved clause if the length is greater than this value
repeat_below: Add another resolved clause for the current quantifier if clause length is greater than this value
iterative: Defines whether to run pre-resolution iteratively on the resolved clauses, and how many iterations to run.
*/
#[derive(Clone)]
pub struct ResolutionConfig {
    pub min_ratio: f32,
    pub max_ratio: f32,
    pub max_clause_length: usize,
    pub repeat_above: usize, 
    pub iterations: i32,
}

/*
An enum to store the type of literal selection.
*/
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LiteralSelection {
    Ordered, // In-order selection
    VariableStateSum, // Variable State Sum selection
}

/*
A struct to store the solver configuration.
*/
#[derive(Clone)]
pub struct Config {
    pub literal_selection: LiteralSelection,
    pub pre_resolution: (bool, ResolutionConfig),
    pub pre_process: bool,
    pub universal_reduction: bool,
    pub pure_literal_deletion: bool,
    pub restarts: bool,
}

impl Config {
    pub fn pure_literal_deletion_enabled(&self) -> bool {
        return self.pure_literal_deletion;
    }

    pub fn universal_reduction_enabled(&self) -> bool {
        return self.universal_reduction;
    }

    pub fn pre_process_enabled(&self) -> bool {
        return self.pre_process;
    }

    pub fn pre_resolution_enabled(&self) -> bool {
        return self.pre_resolution.0;
    }

    pub fn restarts_enabled(&self) -> bool {
        return self.restarts;
    }
}

/*
A struct to store statistics relating to number of unit propagations,
backtrack/backjump counts, and conflict counts where appropriate.
*/
#[derive(Clone)]
pub struct Statistics {
    pub propagation_count: i32,
    pub backtrack_count: i32,
    pub learned_clause_count: i32,
}

impl Statistics {
    /*
    Create an empty statistics struct.
    */
    pub fn new() -> Self {
        Statistics { propagation_count: 0, backtrack_count: 0, learned_clause_count: 0 }
    }

    /*
    A function to increment propagation count.
    */
    pub fn increment_propagation_count(&mut self) {
        self.propagation_count += 1;
    }

    /*
    A function to increment backtrack/backjump count.
    */
    pub fn increment_backtrack_count(&mut self) {
        self.backtrack_count += 1;
    }

    /*
    A function to increment conflict count.
    */
    pub fn increment_learned_clause_count(&mut self) {
        self.learned_clause_count += 1;
    }
}

/*
Structure to store the literals which can be removed by universal reduction, and the clause which they are contained in.
*/
#[derive(Clone)]
pub struct UniversalReductionClause {
    pub clause_index: i32,
    pub values: Vec<i32>,
}

/*
An enum for storing the quantification type.
*/
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum QuantifierType {
    Universal,
    Existential,
}

/*
A struct to store a singular quantified literal that is contained in the quantifier prefix. 

It stores the quantification type, the literal, and the quantification level.
*/
#[derive(Clone, PartialEq)]
pub struct Quantifier {
    pub q_type: QuantifierType,
    pub literal: i32,
    pub q_level: i32,
}

/*
A struct for storing the clause database and the number of non-removed clauses.
*/
#[derive(Clone)]
pub struct ClauseSet {
    pub clause_list: Vec<Clause>,
    pub clause_count: i32,
}

impl ClauseSet {
    /*
    A function to decrease the clause counter one.
    */
    pub fn decrement_counter(&mut self) {
        self.clause_count -= 1;
    }

    /*
    Checks for satisfiability constraint where the empty set exists.
    */
    pub fn contains_empty_set(&self) -> bool {
        return self.clause_count.eq(&0);
    }

    /*
    Checks for unsatisfiability constraint where the empty clause exists.
    */
    pub fn contains_empty_clause(&self) -> bool {
        return self.clause_count.eq(&-1);
    }

    /*
    Checks if a given clause is a contradiction, updates the necessary state variable,
    and returns true if it is, false otherwise.
    */ 
    pub fn check_contradiction(&mut self, clause_index: Option<i32>) -> bool {
        if clause_index.is_none() { 
            if self.clause_count.eq(&-1) {true} else {false}
        } else {
            if self.clause_list[clause_index.unwrap() as usize].is_empty() {
                self.clause_count = -1;
                return true;
            } else {
                return false;
            }
        }
    }
}

/*
A struct for storing a singular clause separated into existential and universal literals which are sorted in the
order in which they appear in the quantifier prefix. The is_removed variable marks whether the clause is removed or not.
*/
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Clause {
    pub e_literals: Vec<i32>, // Sorted into the order the variables occur in the quantifier prefix
    pub a_literals: Vec<i32>, // Sorted into the order the variables occur in the quantifier prefix
    pub is_removed: bool,
}

impl Clause {
    /*
    A function to create a new empty clause.
    */
    pub fn new_empty_clause() -> Self {
        Clause {
            e_literals: Vec::new(),
            a_literals: Vec::new(),
            is_removed: false,
        }
    }

    /*
    A function that checks for a unit clause and returns the unit literal if there is one.
    */
    pub fn is_unit_clause(&self) -> Option<i32> {

        if self.e_literals.len() + self.a_literals.len() == 1  && !self.is_removed {
            if self.a_literals.is_empty() {
                return Some(self.e_literals[0]);
            } else {
                return Some(self.a_literals[0]);
            }
        } else {
            return None;
        }
    }

    /*
    A function to return a list containing all the universal and existential literals within the clause.
    */
    pub fn get_literal_list(self) -> Vec<i32> {
        let mut literals: Vec<i32> = Vec::new();
        literals.extend(self.e_literals);
        literals.extend(self.a_literals);
        return literals;
    }

    /*
    A function to set the a_literals to a given list of literals. Used when reversing universal reduction in CDCL.
    */
    pub fn replace_a_literals(&mut self, literals: Vec<i32>) {
        self.a_literals = literals;
    }

    /*
    A function that removes each universal literal in the given list from the clause.
    */
    pub fn remove_a_literals(&mut self, literals: Vec<i32>) {
        self.a_literals.retain(|&x| !literals.contains(&x));
    }

    /*
    A function that removes the given universal literal from the clause.
    */
    pub fn remove_a_literal(&mut self, literal: i32) {
        self.a_literals.retain(|&x| x != literal);
    }

    /*
    A function that removes the given existential literal from the clause.
    */
    pub fn remove_e_literal(&mut self, literal: i32) {
        self.e_literals.retain(|&x| x != literal);
    }

    /*
    A function to check whether the given clause contains no literals.
    */
    pub fn is_empty(&self) -> bool {
        if self.e_literals.is_empty() && self.a_literals.is_empty() && !self.is_removed {
            return true;
        }
        else {
            return false;
        }
    }

    /*
    A function to get the number of literals in the clause.
    */
    pub fn get_clause_length(&self) -> usize {
        return self.a_literals.len() + self.e_literals.len();
    }
}

/*
A struct for storing information about a given variable/literal. It stores the quantification type, quantification 
level, and value of the literal.
*/
#[derive(Clone)]
pub struct Variable {
    pub q_type: QuantifierType,
    pub q_level: i32,
    pub value: i32,
}

/*
A struct for storing a single assignment of a variable. It stores the value being assigned, the decision level it was 
assigned at, and if applicable the index of the clause that was responsible for causing the given variable to be assigned.
*/
#[derive(Clone)]
pub struct Assignment {
    pub value: i32,
    pub decision_level: i32,
    pub clause_responsible: Option<i32>,
}

impl Assignment {
    /*
    A function to return whether the given assignment was the result of a decision or implication.

    Returns true for a decision, and false for an implication.
    */
    pub fn is_decision(&self) -> bool {
        return self.clause_responsible.is_none();
    }
}

/*
A struct for storing the order in which the literals appeared in the quantifier prefix.
*/
#[derive(Clone)]
pub struct QuantificationOrder {
    pub existential_literal_order: Vec<i32>,
    pub universal_literal_order: Vec<i32>,
}

/*
A struct for storing data needed for facilitating a restart during CDCL.
*/
#[derive(Clone)]
pub struct RestartData {
    pub restart_counter: i32,
    pub conflicts_until_restart: i32,
    pub constant: i32,
    pub current_conflicts: i32,
}

impl RestartData {
    /*
    A function to create a new RestartData data structure.
    */
    pub fn new(constant: i32) -> Self {
        let restart_counter = 1;
        let conflicts_until_restart = constant;
        return RestartData {
            restart_counter, 
            conflicts_until_restart,
            constant,
            current_conflicts: 0,
        };
    }

    /*
    A function to update the number of conflicts that should be allowed before performing a restart. The algorithm
    implements a geometric progression to allow for longer restart intervals based on the luby series.
    */
    pub fn update_conflicts_until_restart(&mut self, restart_count: i32) {
        let fractional_k = (1.0 + restart_count as f32).log2();
        let k = fractional_k.ceil() as u32;
        if fractional_k.fract() == 0.0 {
            self.conflicts_until_restart = self.constant * (2 as i32).pow(k - 1); // When i = 2^k - 1, set to 2^k - 1
        } else {
            let index = restart_count - ((2 as i32).pow(k) / 2) + 1;
            self.update_conflicts_until_restart(index);
        }
    }

    /*
    A function to increase the restart counter by one.
    */
    pub fn increment_restart_counter(&mut self) {
        self.restart_counter += 1;
    }

    /*
    A function to increase the current conflicts by one.
    */
    pub fn increment_current_conflicts(&mut self) {
        self.current_conflicts += 1;
    }

    /*
    A function to reset the current conflicts to zero.
    */
    pub fn reset_current_conflicts(&mut self) {
        self.current_conflicts = 0;
    }
    
    /*
    A function to determine whether a restart should occur or not.

    Returns true if a restart should be performed, and false otherwise.
    */
    pub fn should_restart(&self) -> bool {
        return self.current_conflicts == self.conflicts_until_restart;
    }
}

/*
A struct for storing the core data structures required for performing the DPLL and CDCL procedures.

- quantifier_list stores the quantifier prefix.
- clause_set stores the clause database and clause count.
- clause_references stores the all-watched literals data structure - in a multimap for O(1) access.
- variable_quantification stores the quantification type of each literal - in a multimap for O(1) access.
- quantification_order stores the order in which the literals appear in the quantifier prefix.
- config stores the configuration of the solver stores in config.json.
*/
#[derive(Clone)]
pub struct Matrix {
    pub quantifier_list: Vec<Quantifier>,
    pub clause_set: ClauseSet,
    pub clause_references: MultiMap<i32, i32>,
    pub variable_quantification: MultiMap<i32, Variable>,
    pub quantification_order: QuantificationOrder,
    pub config: Config,
}

impl Matrix {
    /*
    Creates a new Matrix data structure.
    */
    pub fn new(filename: String, config: Config) -> Self {
        let (quantifier_list, clause_set, clause_references, variable_quantification, quantification_order) = Matrix::create_structures(filename);
        return Matrix {
            quantifier_list,
            clause_set,
            clause_references,
            variable_quantification,
            quantification_order,
            config
        };
    }

    /*
    Parses a QBF instance stored in QDIMACS format and generates the data structures required for creating a Matrix.
    */
    pub fn create_structures(filename: String) -> (Vec<Quantifier>, ClauseSet, MultiMap<i32, i32>, MultiMap<i32, Variable>, QuantificationOrder) {
        let mut quantifier_list = Vec::new();
        let mut clause_list = Vec::new();
        let mut clause_references = MultiMap::new();
        let mut variable_quantification = MultiMap::new();

        let mut existential_literal_order = Vec::new();
        let mut universal_literal_order = Vec::new();
        let mut previous_quantifier = String::from("");
        let mut quantification_level = 0;
        let mut clause_count = 0;
        if let Ok(lines) = Matrix::read_lines(filename) {
            for line in lines {
                if let Ok(l) = line {
                    let split = l.split_whitespace();
                    let mut vec = split.clone().collect::<Vec<&str>>();
                    if vec.is_empty() { break };
                    if vec[0].eq("c") || vec[0].eq("p") {
                        continue;
                    } else if vec[0].eq("e") || vec[0].eq("a") {
                        let quantifier_type = vec[0];
                        let quantifier = if quantifier_type.eq("e") {QuantifierType::Existential} else {QuantifierType::Universal};
                        vec.pop();
                        if !quantifier_type.eq(previous_quantifier.as_str()) {
                            previous_quantifier = String::from(quantifier_type);
                            quantification_level += 1;
                        }
                        for &literal in vec.iter().skip(1) { // Skip the quantification element
                            let literal = literal.parse().unwrap();
                            quantifier_list.push(Quantifier {
                                q_type: quantifier.clone(),
                                q_level: quantification_level,
                                literal,
                            });
                            if quantifier_type.eq("e") {
                                existential_literal_order.push(literal);
                            } else {
                                universal_literal_order.push(literal);
                            }
                            variable_quantification.insert(literal, Variable {
                                q_type: quantifier.clone(),
                                q_level: quantification_level,
                                value: literal,
                            })
                        }
                    } else {
                        vec.pop();
                        let mut a_literals = Vec::new();
                        let mut e_literals = Vec::new();
                        for literal in vec {
                            let literal: i32 = literal.parse().unwrap();
                            let negative_literal = -literal;
                            if universal_literal_order.contains(&literal) || universal_literal_order.contains(&negative_literal) {
                                a_literals.push(literal);
                            } else {
                                e_literals.push(literal);
                            }
                            clause_references.insert(literal, clause_count);
                        }

                        a_literals = sort_literals_order(&universal_literal_order, a_literals);
                        e_literals = sort_literals_order(&existential_literal_order, e_literals);

                        clause_list.push(Clause {
                            e_literals,
                            a_literals,
                            is_removed: false,
                        });
                        clause_count += 1;
                    }
                }
            }
        }
        let clause_set = ClauseSet { clause_list, clause_count };
        let quantification_order = QuantificationOrder { existential_literal_order, universal_literal_order };
        return (quantifier_list, clause_set, clause_references, variable_quantification, quantification_order)
    }

    /*
    A function to parse a given file into separate lines.
    */
    pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    /*
    A function that will return true if the current state is either satisfiable (true) or unsatisfiable (false).
    */
    pub fn check_solved(&self) -> bool {
        if self.clause_set.contains_empty_clause() || self.clause_set.contains_empty_set() {
            return true;
        } else {
            return false;
        }
    }
}

/*
A struct for storing the core data structures required for CDCL. Stores the same core structures as DPLL 
with additional ones unique for CDCL. 

- decision_level stores the current decision level the matrix is at in the CDCL procedure.
- conflict_clause stores the clause which caused a given conflict. It's empty if it's not applicable.
- original_clause_list contains the clause_list prior to any modifications.
- trail stores a list of assignments, decisions and implications, in chronological order.
- assignments stores a hashmap of assignments that have been made throughout the CDCL procedure.
- learned_clause_refs stores a list of clause index's which point to which clauses have been learnt.
- restart_data stores the RestartData structure for performing restarts.
*/
#[derive(Clone)]
pub struct CDCLMatrix {
    pub core_data: Matrix,
    pub decision_level: i32,
    pub conflict_clause: Option<Clause>,
    pub original_clause_list: Vec<Clause>,
    pub trail: Vec<Assignment>,
    pub assignments: HashMap<i32, Assignment>,
    pub learned_clause_refs: Vec<i32>,
    pub restart_data: RestartData,
}

impl CDCLMatrix {
    /*
    Creates a new CDCLMatrix data structure.
    */
    pub fn new(filename: String, config: Config) -> Self {
        let core_data = Matrix::new(filename, config);
        let original_clause_list = core_data.clause_set.clause_list.clone();
        return CDCLMatrix {
            core_data,
            decision_level: 0,
            conflict_clause: None,
            original_clause_list,
            trail: Vec::new(),
            assignments: HashMap::new(),
            learned_clause_refs: Vec::new(),
            restart_data: RestartData::new(100),
        };
    }

    /*
    A function to increment the current decision level by one.
    */
    pub fn increment_decision_level(&mut self) {
        self.decision_level += 1;
    }

    /*
    A function to add a learned clause and apply the current assignments. It will update necessary structures for keeping
    track of clause count and clause references.
    */
    pub fn add_clause(&mut self, clause: &Clause) {
        // Push original clause to the original clause store.
        self.original_clause_list.push(clause.clone());

        // Apply the current assignments to the clause and update necessary attributes.
        let new_clause = self.apply_current_assignments(clause);
        self.core_data.clause_set.clause_list.push(new_clause.clone());
        
        let clause_index = self.core_data.clause_set.clause_list.len() - 1;
        self.learned_clause_refs.push(clause_index as i32);
        for literal in new_clause.get_literal_list() {
            self.core_data.clause_references.insert(literal, clause_index as i32)
        }
        self.core_data.clause_set.clause_count += 1;
    }

    /*
    A function to apply the current assignments that have been made so far in the decision tree to a given clause.
    */
    pub fn apply_current_assignments(&self, clause: &Clause) -> Clause {
        let mut new_clause = clause.clone();
        for e_literal in &clause.e_literals {
            if !self.assignments.get(&e_literal.abs()).is_none() {
                new_clause.remove_e_literal(*e_literal);
            }
        }
        for a_literal in &clause.a_literals {
            if !self.assignments.get(&a_literal.abs()).is_none() {
                new_clause.remove_a_literal(*a_literal);
            }
        }
        return new_clause;
    }
    
    /*
    A functio that will re-add learned clauses to the clause database. This is needed when restoring cached data structures
    which don't hold newly learned clauses.
    */
    pub fn readd_learned_clauses(&mut self) {
        for reference in &self.learned_clause_refs {
            if reference > &(self.core_data.clause_set.clause_list.len() as i32 - 1) {
                let mut clause = self.original_clause_list[*reference as usize].clone();
                clause = self.apply_current_assignments(&clause);
                self.core_data.clause_set.clause_list.push(clause.clone());
                for literal in clause.get_literal_list() {
                    self.core_data.clause_references.insert(literal, (self.core_data.clause_set.clause_list.len() - 1) as i32)
                }
                self.core_data.clause_set.clause_count += 1;
            }
        }
    }

    /*
    A function to reduce the clause database by 50% by applying age-based deletion.
    */
    pub fn reduce_clause_database(&mut self) {
        let num_of_learned_clauses = &self.learned_clause_refs.len();
        let first_half = self.learned_clause_refs[0 .. (num_of_learned_clauses / 2)].to_vec();
        // Remove from clause_list  and remove from original clause_set
        for reference in first_half.iter().rev() {
            self.original_clause_list.remove(*reference as usize);
            self.core_data.clause_set.clause_list.remove(*reference as usize);
            self.learned_clause_refs.remove(0);
            self.core_data.clause_set.clause_count -= 1;
        }
        self.refresh_clause_references();
        for reference in self.learned_clause_refs.iter_mut() {
            *reference -= first_half.len() as i32;
        }
    }
    
    /*
    A function to update the clause references in the clause database.
    */
    pub fn refresh_clause_references(&mut self) {
        let mut clause_references = MultiMap::new();
        for (index, clause) in self.core_data.clause_set.clause_list.iter().enumerate() {
            for literal in clause.clone().get_literal_list() {
                clause_references.insert(literal, index as i32);
            }
        }
        self.core_data.clause_references = clause_references;
    }

    /*
    A function to remove the conflict clause when it's no longer needed.
    */
    pub fn reset_conflict_clause(&mut self) {
        self.conflict_clause = None;
    }
}