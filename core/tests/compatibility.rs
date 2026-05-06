use jsonlogic_fast::evaluate;
use serde_json::json;

#[test]
fn test_official_compatibility() {
    // Basic arithmetic
    assert_eq!(evaluate(r#"{"*": [2, 3]}"#, r#"{}"#).unwrap(), json!(6.0));
    assert_eq!(evaluate(r#"{"/": [6, 2]}"#, r#"{}"#).unwrap(), json!(3.0));
    assert_eq!(evaluate(r#"{"%": [7, 2]}"#, r#"{}"#).unwrap(), json!(1.0));
    assert_eq!(evaluate(r#"{"+": [2, 3]}"#, r#"{}"#).unwrap(), json!(5.0));
    assert_eq!(evaluate(r#"{"-": [3, 2]}"#, r#"{}"#).unwrap(), json!(1.0));

    // Logic and boolean
    assert_eq!(evaluate(r#"{"==": [1, 1]}"#, r#"{}"#).unwrap(), json!(true));
    assert_eq!(
        evaluate(r#"{"==": [1, "1"]}"#, r#"{}"#).unwrap(),
        json!(true)
    );
    assert_eq!(evaluate(r#"{"!=": [1, 2]}"#, r#"{}"#).unwrap(), json!(true));
    assert_eq!(evaluate(r#"{"!": true}"#, r#"{}"#).unwrap(), json!(false));
    assert_eq!(evaluate(r#"{"!!": "foo"}"#, r#"{}"#).unwrap(), json!(true));

    // Comparisons
    assert_eq!(evaluate(r#"{">": [2, 1]}"#, r#"{}"#).unwrap(), json!(true));
    assert_eq!(evaluate(r#"{">=": [2, 2]}"#, r#"{}"#).unwrap(), json!(true));
    assert_eq!(evaluate(r#"{"<": [1, 2]}"#, r#"{}"#).unwrap(), json!(true));
    assert_eq!(evaluate(r#"{"<=": [2, 2]}"#, r#"{}"#).unwrap(), json!(true));

    // Data structures and iteration
    assert_eq!(
        evaluate(r#"{"var": ["a"]}"#, r#"{"a": 1}"#).unwrap(),
        json!(1)
    );
    assert_eq!(
        evaluate(r#"{"var": ["a.b"]}"#, r#"{"a": {"b": 2}}"#).unwrap(),
        json!(2)
    );
    assert_eq!(
        evaluate(r#"{"missing": ["a", "b"]}"#, r#"{"a": 1}"#).unwrap(),
        json!(["b"])
    );
    assert_eq!(
        evaluate(r#"{"missing_some": [1, ["a", "b", "c"]]}"#, r#"{"a": 1}"#).unwrap(),
        json!([])
    );

    // Array operations
    assert_eq!(
        evaluate(r#"{"in": ["Spring", "Springfield"]}"#, r#"{}"#).unwrap(),
        json!(true)
    );
    assert_eq!(
        evaluate(r#"{"in": [1, [1, 2, 3]]}"#, r#"{}"#).unwrap(),
        json!(true)
    );
    assert_eq!(
        evaluate(r#"{"cat": ["a", "b", "c"]}"#, r#"{}"#).unwrap(),
        json!("abc")
    );

    // Higher order
    assert_eq!(
        evaluate(
            r#"{"map": [{"var":"x"}, {"*":[{"var":""}, 2]}]}"#,
            r#"{"x": [1, 2, 3]}"#
        )
        .unwrap(),
        json!([2.0, 4.0, 6.0])
    );
    assert_eq!(
        evaluate(
            r#"{"filter": [{"var":"x"}, {">":[{"var":""}, 1]}]}"#,
            r#"{"x": [1, 2, 3]}"#
        )
        .unwrap(),
        json!([2, 3])
    );
    assert_eq!(
        evaluate(
            r#"{"reduce": [{"var":"x"}, {"+":[{"var":"current"}, {"var":"accumulator"}]}, 0]}"#,
            r#"{"x": [1, 2, 3]}"#
        )
        .unwrap(),
        json!(6.0)
    );
    assert_eq!(
        evaluate(r#"{"merge": [[1, 2], [3, 4]]}"#, r#"{}"#).unwrap(),
        json!([1, 2, 3, 4])
    );
}
