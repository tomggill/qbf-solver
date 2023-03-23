#[cfg(test)]
mod test {
    use multimap::MultiMap;
    use serde_json::json;

    use crate::{universal_reduction::{get_universal_literals_for_reduction, remove_universal_literal, detect_universal_literal}, data_structures::{Matrix, QuantifierType, Variable, Clause, ResolutionConfig, LiteralSelection, Config, SolverType, Quantifier}, pure_literal_deletion::{get_pure_literals, remove_pure_literals}, resolution::{resolve, add_resolved_clauses, pre_resolution}, util::{convert_literals_to_clause, read_instance_name, get_unit_literals, get_quantifier_type, get_variable_state_sum, sort_literals_order}, parse_config::{read_number_json_f32, read_number_json_usize, read_number_json_i32, read_boolean_json, read_string_json, read_solver_type_json, read_literal_selection_json}, literal_selection::{select_literal, select_literal_vss}};


    fn config() -> Config {
        Config {
            literal_selection: LiteralSelection::Ordered,
            pre_resolution: (false, ResolutionConfig {
                min_ratio: 0.25,
                max_ratio: 0.5,
                max_clause_length: usize::MAX,
                repeat_above: 3,
                iterations: 1,
            }),
            pre_process: true,
            universal_reduction: true,
            pure_literal_deletion: true,
            restarts: false,
        }
    }
    
    /* START OF UNIVERSAL REDUCTION TESTS */

    /*
    Tests that universal reduction detects empty clauses.
    */
    #[test]
    fn unsatisfiable_by_universal_reduction_test() {
        let filename = "./test_files/universal_reduction_test.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let universal_reductions = get_universal_literals_for_reduction(&matrix.clause_set.clause_list, &matrix.variable_quantification);
        for reduction in universal_reductions {
            remove_universal_literal(matrix, reduction.values, reduction.clause_index);
        }
        assert_eq!(-1, matrix.clause_set.clause_count);
    }

    /*
    Tests that literals that can be reduced by universal reduction are detected correctly.
    */
    #[test]
    fn detect_universal_literal_test() {
        let clause = Clause { e_literals: vec![1,2], a_literals: vec![3], is_removed: false };
        let mut variable_quantification = MultiMap::new();
        variable_quantification.insert(1, Variable { q_type: QuantifierType::Existential, q_level: 1, value: 1 });
        variable_quantification.insert(2, Variable { q_type: QuantifierType::Existential, q_level: 1, value: 2 });
        variable_quantification.insert(3, Variable { q_type: QuantifierType::Universal, q_level: 2, value: 3 });
        let detected_universal_literals_for_reduction = detect_universal_literal(&clause, &variable_quantification);
        assert_eq!(3, detected_universal_literals_for_reduction[0]);
    }

    /*
    Tests that literals that can be reduced by universal reduction are removed correctly.
    */
    #[test]
    pub fn remove_universal_literal_test() {
        let filename = "./test_files/universal_reduction_test2.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let universal_reductions = get_universal_literals_for_reduction(&matrix.clause_set.clause_list, &matrix.variable_quantification);
        for reduction in universal_reductions {
            remove_universal_literal(matrix, reduction.values, reduction.clause_index);
        }
        let clause_to_check = matrix.clause_set.clause_list[2].clone();
        assert_ne!(None, clause_to_check.is_unit_clause());
        assert_eq!(vec![1], clause_to_check.e_literals);
    }
    /* END OF UNIVERSAL REDUCTION TESTS */

    /* START OF PURE LITERAL DELETION TESTS */

    /*
    Tests that pure literals are detected correctly.
    */
    #[test]
    pub fn get_pure_literals_test() {
        let mut clause_references = MultiMap::new();
        clause_references.insert(1, 0);
        clause_references.insert(2, 0);
        clause_references.insert(-2, 1);
        clause_references.insert(-3, 2);
        let pure_literals = get_pure_literals(&clause_references);
        assert!(pure_literals.contains(&-3));
        assert!(pure_literals.contains(&1));
    }

    /*
    Tests that pure literals are removed correctly.
    */
    #[test]
    pub fn remove_pure_literals_test() {
        let filename = "./test_files/pure_literal_removal_test.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let pure_literals = get_pure_literals(&matrix.clause_references);
        let found_unit_literals = remove_pure_literals(matrix, pure_literals);
        assert_eq!(3, found_unit_literals[0]);
        assert_eq!(1, matrix.clause_set.clause_count);
    }
    /* END OF PURE LITERAL DELETION TESTS */

    /* START OF RESOLUTION TESTS */

    /*
    Tests that the resolve functionality can detect unsound resolutions.
    */
    #[test]
    pub fn invalid_resolve_test() {
        let literals_1 = vec![1,2,3];
        let literals_2 = vec![-1,-2,6];
        let literal = 1;
        let resolution = resolve(literals_1, literals_2, literal);
        assert_eq!(true, resolution.is_none());
    }

    /*
    Tests that the resolve functionality can perform Q-Resolution correctly.
    */
    #[test]
    pub fn valid_resolve_test() {
        let literals_1 = vec![1,2,3];
        let literals_2 = vec![-1,4,6];
        let literal = 1;
        let mut resolution = resolve(literals_1, literals_2, literal).unwrap();
        resolution.sort();
        assert_eq!(vec![2,3,4,6], resolution);
    }

    /*
    Tests that resolved clauses are added to the clause database correctly.
    */
    #[test]
    pub fn add_resolved_clauses_test() {
        let filename = "./test_files/preresolution_test.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let resolved_clause = convert_literals_to_clause(&matrix.variable_quantification, &matrix.quantification_order, &vec![2,3]);
        add_resolved_clauses(matrix, vec![resolved_clause.clone()], 3, &mut Vec::new());
        assert_eq!(3, matrix.clause_set.clause_count);
        assert_eq!(matrix.clause_set.clause_list[2], resolved_clause);
    }

    /*
    Tests that pre-resolution is performed correctly.
    */
    #[test]
    pub fn pre_resolution_test() {
        let filename = "./test_files/preresolution_test.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        pre_resolution(matrix, &mut Vec::new());
        assert_eq!(3, matrix.clause_set.clause_count);
        assert_eq!(vec![2,3], matrix.clause_set.clause_list[2].clone().get_literal_list());
    }
    /* END OF RESOLUTION TESTS */

    /* START OF LITERAL SELECTION TESTS */

    /*
    Tests that the literals are selected in order and variable quantification is obtained.
    */
    #[test]
    pub fn ordered_literal_selection_test_1() {
        let filename = "./test_files/ordered_literal_selection_test.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let (literal, quantifier_type) = select_literal(matrix);
        assert_eq!(2, literal);
        assert_eq!(QuantifierType::Existential, quantifier_type);
    }

    /*
    Tests that the literals are selected in order and void quantifiers are ignored and removed.
    */
    #[test]
    pub fn ordered_literal_selection_test_2() {
        let filename = "./test_files/ordered_literal_selection_test.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        select_literal(matrix);
        let void_quantifier = Quantifier {
            q_type: QuantifierType::Existential,
            literal: 1,
            q_level: 1,
        };
        assert_eq!(false, matrix.quantifier_list.contains(&void_quantifier));
    }

    /* 
    Tests that the literals are selected using variable state sum and void quantifiers are ignored and removed.
    */
    #[test]
    pub fn variable_state_sum_selection_test_1() {
        let filename = "./test_files/ordered_literal_selection_test.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let (literal, quantifier_type) = select_literal_vss(matrix);
        assert_eq!(3, literal);
        assert_eq!(QuantifierType::Existential, quantifier_type);

        let void_quantifier = Quantifier {
            q_type: QuantifierType::Existential,
            literal: 1,
            q_level: 1,
        };
        assert_eq!(false, matrix.quantifier_list.contains(&void_quantifier));
    }

    /* END OF LITERAL SELECTION TESTS */

    /* START OF UTIL TESTS */

    /*
    Tests that literals are sorted in the correct order according to the order they appear in the quantifier prefix.
    */
    #[test]
    pub fn sort_literals_order_test() {
        let filename = "./test_files/sort_literals_order_test.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let literals = vec![7,2,3,1];
        let sorted_literals = sort_literals_order(&matrix.quantification_order.existential_literal_order, literals);
        assert_eq!(vec![1,2,3,7], sorted_literals);
    }

    /*
    Tests that the variable state sum value is correct.
    */
    #[test]
    pub fn get_variable_state_sum_test() {
        let filename = "./test_files/get_variable_state_sum_test.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let (vss, positive_sign) = get_variable_state_sum(&matrix.clause_references, 1);
        assert_eq!(3, vss);
        assert_eq!(true, positive_sign);
    }

    /*
    Tests that literals are converted to a properly formatted clause within covert_literals_to_clause.
    */
    #[test]
    pub fn convert_literals_to_clause_test() {
        let filename = "./test_files/convert_literals_to_clause_test.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let converted_clause = convert_literals_to_clause(&matrix.variable_quantification, &matrix.quantification_order, &vec![3, 2, 4, 1]);
        let expected_clause = Clause {
            e_literals: vec![1, 2, 3],
            a_literals: vec![4],
            is_removed: false,
        };
        assert_eq!(expected_clause, converted_clause);
    }

    /*
    Tests that the quantifier type and index is found correctly when it exists in the quantifier prefix.
    */
    #[test]
    pub fn get_quantifier_type_test_1() {
        let filename = "./test_files/get_quantifier_type_test1.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let (quantifier_type, quantifier_index) = get_quantifier_type(&matrix.quantifier_list, 1);
        assert_eq!(QuantifierType::Existential, quantifier_type);
        assert_eq!(false, quantifier_index.is_none());
        assert_eq!(0, quantifier_index.unwrap());
    }

    /*
    Tests that quantifier type existential and no index is returned for a literal not appearing in the quanitifer prefix.
    */
    #[test]
    pub fn get_quantifier_type_test_2() {
        let filename = "./test_files/get_quantifier_type_test2.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let (quantifier_type, quantifier_index) = get_quantifier_type(&matrix.quantifier_list, 4);
        assert_eq!(QuantifierType::Existential, quantifier_type);
        assert_eq!(true, quantifier_index.is_none());
    }

    /*
    Tests that unit literals are found from the clause database correctly.
    */
    #[test]
    pub fn get_unit_literals_test_1() {
        let filename = "./test_files/get_unit_literals_test1.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let mut unit_literals = get_unit_literals(&matrix.clause_set.clause_list);
        unit_literals.sort();
        assert_eq!(vec![2,4], unit_literals);

    }

    /*
    Tests that when no unit literals exist, none are found.
    */
    #[test]
    pub fn get_unit_literals_test_2() {
        let filename = "./test_files/get_unit_literals_test2.qdimacs".to_string();
        let matrix = &mut Matrix::new(filename, config());
        let unit_literals = get_unit_literals(&matrix.clause_set.clause_list);
        assert_eq!(true, unit_literals.is_empty());

    }

    /*
    Tests that during running benchmarks, the instance name of a file in qdimacs form is extracted properly.
    */
    #[test]
    pub fn read_instance_name_test() {
        let file_path= "./benchmarks/castellini\\toilet_a_02_01.2.qdimacs".to_string();
        let instance_name = read_instance_name(&file_path);
        let expected_instance_name = "toilet_a_02_01.2.qdimacs".to_string();
        assert_eq!(expected_instance_name, instance_name);
    }

    /* END OF UTIL TESTS */

    /* START OF CONFIG PARSER TESTS */

    /*
    Tests reading floats returns a float value when parsing a float.
    */
    #[test]
    pub fn read_floats_valid_test_1() {
        let json_values = json!({"number": 0.25});
        let float_value = read_number_json_f32(&json_values["number"]);
        assert_eq!(false, float_value.is_none());
        assert_eq!(0.25, float_value.unwrap());
    }

    /*
    Tests reading floats returns a float value when parsing an integer.
    */
    #[test]
    pub fn read_floats_valid_test_2() {
        let json_values = json!({"number": 2});
        let float_value = read_number_json_f32(&json_values["number"]);
        assert_eq!(false, float_value.is_none());
        assert_eq!(2.0, float_value.unwrap());
    }

    /*
    Tests reading floats returns a max float value when parsing an infinity string value.
    */
    #[test]
    pub fn read_floats_infinity_test() {
        let json_values = json!({"number": "infinity"});
        let float_value = read_number_json_f32(&json_values["number"]);
        assert_eq!(false, float_value.is_none());
        assert_eq!(f32::MAX, float_value.unwrap());
    }

    /*
    Tests reading floats does not allow strings other than infinity.
    */
    #[test]
    pub fn read_floats_invalid_test_1() {
        let json_values = json!({"number": "string..."});
        let float_value = read_number_json_f32(&json_values["number"]);
        assert_eq!(true, float_value.is_none());
    }

    /*
    Tests reading floats does not allow Boolean values.
    */
    #[test]
    pub fn read_floats_invalid_test_2() {
        let json_values = json!({"number": false});
        let float_value = read_number_json_f32(&json_values["number"]);
        assert_eq!(true, float_value.is_none());
    }

    /*
    Tests reading unsigned integers returns a usize value when parsing an integer.
    */
    #[test]
    pub fn read_usize_valid_test_1() {
        let json_values = json!({"number": 3});
        let usize_value = read_number_json_usize(&json_values["number"]);
        assert_eq!(false, usize_value.is_none());
        assert_eq!(3 as usize, usize_value.unwrap());
    }

    /*
    Tests reading unsigned integers does not allow floats.
    */
    #[test]
    pub fn read_usize_valid_test_2() {
        let json_values = json!({"number": 0.25});
        let usize_value = read_number_json_usize(&json_values["number"]);
        assert_eq!(true, usize_value.is_none());
    }

    /*
    Tests reading unsigned integers returns a max usize value when parsing an infinity string value.
    */
    #[test]
    pub fn read_usize_infinity_test() {
        let json_values = json!({"number": "infinity"});
        let usize_value = read_number_json_usize(&json_values["number"]);
        assert_eq!(false, usize_value.is_none());
        assert_eq!(usize::MAX, usize_value.unwrap());
    }

    /*
    Tests reading unsigned integers does not allow strings other than infinity.
    */
    #[test]
    pub fn read_usize_invalid_test_1() {
        let json_values = json!({"number": "string..."});
        let usize_value = read_number_json_usize(&json_values["number"]);
        assert_eq!(true, usize_value.is_none());
    }

    /*
    Tests reading unsigned integers does not allow Boolean values.
    */
    #[test]
    pub fn read_usize_invalid_test_2() {
        let json_values = json!({"number": false});
        let usize_value = read_number_json_usize(&json_values["number"]);
        assert_eq!(true, usize_value.is_none());
    }

    /*
    Tests reading integers returns an i32 value when reading an integer.
    */
    #[test]
    pub fn read_integer_valid_test_1() {
        let json_values = json!({"number": 5});
        let integer_value = read_number_json_i32(&json_values["number"]);
        assert_eq!(false, integer_value.is_none());
        assert_eq!(5 as i32, integer_value.unwrap());
    }

    /*
    Tests reading integers does not allow floats.
    */
    #[test]
    pub fn read_integer_valid_test_2() {
        let json_values = json!({"number": 0.5});
        let integer_value = read_number_json_i32(&json_values["number"]);
        assert_eq!(true, integer_value.is_none());
    }

    /*
    Tests reading integers does not allow infinity strings.
    */
    #[test]
    pub fn read_integer_infinity_invalid_test() {
        let json_values = json!({"number": "infinity"});
        let integer_value = read_number_json_i32(&json_values["number"]);
        assert_eq!(true, integer_value.is_none());
    }

    /*
    Tests reading integers does not allow string values.
    */
    #[test]
    pub fn read_integer_invalid_test_1() {
        let json_values = json!({"number": "string..."});
        let integer_value = read_number_json_i32(&json_values["number"]);
        assert_eq!(true, integer_value.is_none());
    }

    /*
    Tests reading integers does not allow Boolean values.
    */
    #[test]
    pub fn read_integer_invalid_test_2() {
        let json_values = json!({"number": false});
        let integer_value = read_number_json_i32(&json_values["number"]);
        assert_eq!(true, integer_value.is_none());
    }

    /*
    Testing reading Boolean values returns a Boolean value.
    */
    #[test]
    pub fn read_boolean_valid_test_1() {
        let json_values = json!({"boolean": true});
        let bool_value = read_boolean_json(&json_values["boolean"]);
        assert_eq!(false, bool_value.is_none());
        assert_eq!(true, bool_value.unwrap());
    }

    /*
    Testing reading Boolean values does not allow integer values.
    */
    #[test]
    pub fn read_boolean_invalid_test_1() {
        let json_values = json!({"boolean": 5});
        let bool_value = read_boolean_json(&json_values["boolean"]);
        assert_eq!(true, bool_value.is_none());
    }

    /*
    Testing reading Boolean values does not allow string values.
    */
    #[test]
    pub fn read_boolean_invalid_test_2() {
        let json_values = json!({"boolean": "string..."});
        let bool_value = read_boolean_json(&json_values["boolean"]);
        assert_eq!(true, bool_value.is_none());
    }

    /*
    Testing reading string values returns a string value.
    */
    #[test]
    pub fn read_string_valid_test_1() {
        let json_values = json!({"string": "string..."});
        let integer_value = read_string_json(&json_values["string"]);
        assert_eq!(false, integer_value.is_none());
        assert_eq!("string...".to_string(), integer_value.unwrap());
    }

    /*
    Testing reading string values does not allow integers.
    */
    #[test]
    pub fn read_string_invalid_test_1() {
        let json_values = json!({"string": 5});
        let integer_value = read_string_json(&json_values["string"]);
        assert_eq!(true, integer_value.is_none());
    }

    /*
    Testing reading solver type allows "CDCL".
    */
    #[test]
    pub fn read_solver_type_valid_test_1() {
        let json_values = json!({"SolverType": "CDCL"});
        let solver_type_value = read_solver_type_json(&json_values["SolverType"]);
        assert_eq!(false, solver_type_value.is_none());
        assert_eq!(SolverType::CDCL, solver_type_value.unwrap());
    }

    /*
    Testing reading solver type allows "DPLL".
    */
    #[test]
    pub fn read_solver_type_valid_test_2() {
        let json_values = json!({"SolverType": "dpll"});
        let solver_type_value = read_solver_type_json(&json_values["SolverType"]);
        assert_eq!(false, solver_type_value.is_none());
        assert_eq!(SolverType::DPLL, solver_type_value.unwrap());
    }

    /*
    Testing reading solver type does not allow any other string.
    */
    #[test]
    pub fn read_solver_type_invalid_test() {
        let json_values = json!({"SolverType": "invalid-solver"});
        let solver_type_value = read_solver_type_json(&json_values["SolverType"]);
        assert_eq!(true, solver_type_value.is_none());
    }

    /*
    Testing reading literal selection type allows "VSS".
    */
    #[test]
    pub fn read_literal_selection_type_valid_test_1() {
        let json_values = json!({"LiteralSelection": "VSS"});
        let literal_selection_value = read_literal_selection_json(&json_values["LiteralSelection"]);
        assert_eq!(false, literal_selection_value.is_none());
        assert_eq!(LiteralSelection::VariableStateSum, literal_selection_value.unwrap());
    }

    /*
    Testing reading literal selection type allows "Ordered".
    */
    #[test]
    pub fn read_literal_selection_type_valid_test_2() {
        let json_values = json!({"LiteralSelection": "ordered"});
        let literal_selection_value = read_literal_selection_json(&json_values["LiteralSelection"]);
        assert_eq!(false, literal_selection_value.is_none());
        assert_eq!(LiteralSelection::Ordered, literal_selection_value.unwrap());
    }

    /*
    Testing reading literal selection type does not allow any other string.
    */
    #[test]
    pub fn read_literal_selection_type_invalid_test() {
        let json_values = json!({"LiteralSelection": "literal-selection-type"});
        let literal_selection_value = read_literal_selection_json(&json_values["LiteralSelection"]);
        assert_eq!(true, literal_selection_value.is_none());
    }

    /* END OF CONFIG PARSER TESTS */
}