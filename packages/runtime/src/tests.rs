use dioscript_parser::ast::DioscriptAst;

use crate::core::error::RuntimeError;
use crate::core::types::ElementContentType;
use crate::{Executor, Value};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------
fn exec(code: &str) -> Result<Value, RuntimeError> {
    let ast = DioscriptAst::from_string(code).expect("Parse should succeed");
    let mut executor = Executor::init();
    executor.execute(ast)
}

fn exec_ok(code: &str) -> Value {
    exec(code).expect("Execution should succeed")
}

// ===========================================================================
// 1. Arithmetic operations
// ===========================================================================

#[test]
fn test_add() {
    assert_eq!(exec_ok("return 1 + 2;"), Value::Number(3.0));
}

#[test]
fn test_sub() {
    assert_eq!(exec_ok("return 3 - 1;"), Value::Number(2.0));
}

#[test]
fn test_mul() {
    assert_eq!(exec_ok("return 4 * 2;"), Value::Number(8.0));
}

#[test]
fn test_div() {
    assert_eq!(exec_ok("return 8 / 2;"), Value::Number(4.0));
}

#[test]
fn test_mod() {
    assert_eq!(exec_ok("return 7 % 3;"), Value::Number(1.0));
}

// ===========================================================================
// 2. String concatenation
// ===========================================================================

#[test]
fn test_string_concat() {
    assert_eq!(
        exec_ok(r#"return "hello" + " world";"#),
        Value::String("hello world".to_string())
    );
}

#[test]
fn test_string_number_concat() {
    assert_eq!(
        exec_ok(r#"return "value: " + 42;"#),
        Value::String("value: 42".to_string())
    );
}

// ===========================================================================
// 3. Comparison operators
// ===========================================================================

#[test]
fn test_eq_true() {
    assert_eq!(exec_ok("return 1 == 1;"), Value::Boolean(true));
}

#[test]
fn test_eq_false() {
    assert_eq!(exec_ok("return 1 == 2;"), Value::Boolean(false));
}

#[test]
fn test_ne_true() {
    assert_eq!(exec_ok("return 1 != 2;"), Value::Boolean(true));
}

#[test]
fn test_gt_true() {
    assert_eq!(exec_ok("return 3 > 2;"), Value::Boolean(true));
}

#[test]
fn test_lt_true() {
    assert_eq!(exec_ok("return 2 < 3;"), Value::Boolean(true));
}

#[test]
fn test_ge_true() {
    assert_eq!(exec_ok("return 2 >= 2;"), Value::Boolean(true));
}

#[test]
fn test_le_true() {
    assert_eq!(exec_ok("return 1 <= 2;"), Value::Boolean(true));
}

// ===========================================================================
// 4. Logical operators
// ===========================================================================

#[test]
fn test_and_true() {
    assert_eq!(exec_ok("return true && true;"), Value::Boolean(true));
}

#[test]
fn test_and_false() {
    assert_eq!(exec_ok("return true && false;"), Value::Boolean(false));
}

#[test]
fn test_or_true() {
    assert_eq!(exec_ok("return false || true;"), Value::Boolean(true));
}

#[test]
fn test_or_false() {
    assert_eq!(exec_ok("return false || false;"), Value::Boolean(false));
}

// ===========================================================================
// 5. Variables
// ===========================================================================

#[test]
fn test_variable_define_and_return() {
    assert_eq!(exec_ok("let x = 10; return x;"), Value::Number(10.0));
}

#[test]
fn test_variable_reassign() {
    assert_eq!(exec_ok("let x = 5; x = 10; return x;"), Value::Number(10.0));
}

// ===========================================================================
// 6. Conditional statements
// ===========================================================================

#[test]
fn test_if_true() {
    assert_eq!(exec_ok("if true { return 1; }"), Value::Number(1.0));
}

#[test]
fn test_if_false_no_else() {
    assert_eq!(exec_ok("if false { return 1; }"), Value::None);
}

#[test]
fn test_if_false_else() {
    assert_eq!(
        exec_ok("if false { return 1; } else { return 2; }"),
        Value::Number(2.0)
    );
}

// ===========================================================================
// 7. Loops (while)
// ===========================================================================

#[test]
fn test_while_loop_sum() {
    assert_eq!(
        exec_ok("let sum = 0; while sum < 3 { sum = sum + 1; } return sum;"),
        Value::Number(3.0)
    );
}

// ===========================================================================
// 8. Functions
// ===========================================================================

#[test]
fn test_function_call() {
    // Named function definitions are now correctly handled (no double-definition bug).
    assert_eq!(
        exec_ok("fn add(a, b) { return a + b; } return add(3, 4);"),
        Value::Number(7.0)
    );
}

#[test]
fn test_function_no_params() {
    assert_eq!(
        exec_ok("fn answer() { return 42; } return answer();"),
        Value::Number(42.0)
    );
}

// ===========================================================================
// 9. For-in loop
// ===========================================================================

#[test]
fn test_for_in_loop() {
    assert_eq!(
        exec_ok("let arr = [1, 2, 3]; let sum = 0; for i in arr { sum = sum + i; } return sum;"),
        Value::Number(6.0)
    );
}

// ===========================================================================
// 10. List indexing
// ===========================================================================

#[test]
fn test_list_index() {
    assert_eq!(
        exec_ok("let arr = [1, 2, 3]; return arr[1];"),
        Value::Number(2.0)
    );
}

#[test]
fn test_list_index_zero() {
    assert_eq!(
        exec_ok("let arr = [1, 2, 3]; return arr[0];"),
        Value::Number(1.0)
    );
}

// ===========================================================================
// 11. Dictionary indexing
// ===========================================================================

#[test]
fn test_dict_index() {
    assert_eq!(
        exec_ok(r#"let d = { "x": 10, "y": 20 }; return d["x"];"#),
        Value::Number(10.0)
    );
}

// ===========================================================================
// 12. Element generation
// ===========================================================================

#[test]
fn test_element_simple() {
    let result = exec_ok(r#"return div { "hello", class: "main" };"#);
    match result {
        Value::Element(el) => {
            assert_eq!(el.name, "div");
            assert_eq!(
                el.attributes.get("class"),
                Some(&Value::String("main".to_string()))
            );
            assert_eq!(el.content.len(), 1);
            assert_eq!(
                el.content[0],
                ElementContentType::Content("hello".to_string())
            );
        }
        other => panic!("Expected Element, got {:?}", other),
    }
}

#[test]
fn test_element_nested() {
    let result = exec_ok(r#"return div { class: "outer", p { "inner" } };"#);
    match result {
        Value::Element(el) => {
            assert_eq!(el.name, "div");
            assert_eq!(
                el.attributes.get("class"),
                Some(&Value::String("outer".to_string()))
            );
            assert_eq!(el.content.len(), 1);
            match &el.content[0] {
                ElementContentType::Children(child) => {
                    assert_eq!(child.name, "p");
                    assert_eq!(child.content.len(), 1);
                    assert_eq!(
                        child.content[0],
                        ElementContentType::Content("inner".to_string())
                    );
                }
                other => panic!("Expected Children, got {:?}", other),
            }
        }
        other => panic!("Expected Element, got {:?}", other),
    }
}

// ===========================================================================
// 13. Error cases
// ===========================================================================

#[test]
fn test_negative_index_error() {
    let result = exec("let arr = [1, 2, 3]; return arr[-1];");
    assert!(result.is_err(), "Negative index should return an error");
}

#[test]
fn test_out_of_bounds_index_error() {
    let result = exec("let arr = [1, 2, 3]; return arr[5];");
    assert!(
        result.is_err(),
        "Out-of-bounds index should return an error"
    );
}

#[test]
fn test_div_zero_error() {
    let result = exec("return 1 / 0;");
    assert!(result.is_err(), "Division by zero should return an error");
}

#[test]
fn test_type_too_few_args_error() {
    let result = exec("return type();");
    assert!(
        result.is_err(),
        "type() with zero arguments should return an error"
    );
}

#[test]
fn test_variable_not_found_error() {
    let result = exec("return undefined_var;");
    assert!(result.is_err(), "Undefined variable should return an error");
}

// ===========================================================================
// 14. Nested functions / closures
// ===========================================================================

#[test]
fn test_nested_function_closure() {
    // Named functions in DioScript use dynamic scope lookup, not lexical capture.
    // Closures (anonymous functions) capture free variables at definition time.
    assert_eq!(
        exec_ok("fn outer() { let x = 10; let inner = fn () { return x; }; return inner(); } return outer();"),
        Value::Number(10.0)
    );
}

// ===========================================================================
// 15. Built-in functions
// ===========================================================================

#[test]
fn test_type_number() {
    assert_eq!(
        exec_ok("return type(42);"),
        Value::String("number".to_string())
    );
}

#[test]
fn test_type_string() {
    assert_eq!(
        exec_ok(r#"return type("hi");"#),
        Value::String("string".to_string())
    );
}

#[test]
fn test_type_boolean() {
    assert_eq!(
        exec_ok("return type(true);"),
        Value::String("boolean".to_string())
    );
}

#[test]
fn test_type_list() {
    assert_eq!(
        exec_ok("return type([1, 2, 3]);"),
        Value::String("list".to_string())
    );
}
