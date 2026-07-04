#![allow(dead_code)]

pub mod element;
pub mod parser;
pub mod types;

pub mod error;

pub mod ast;

#[cfg(test)]
mod tests {
    use crate::ast::{DioAstStatement, DioscriptAst, FunctionName, LoopExecuteType};
    use crate::element::AstElementContentType;
    use crate::parser::CalcExpr;
    use crate::types::AstValue;

    // ---------------------------------------------------------------------------
    // Helper
    // ---------------------------------------------------------------------------
    fn parse(source: &str) -> DioscriptAst {
        DioscriptAst::from_string(source).expect("Parse should succeed")
    }

    // ===========================================================================
    // 1. Variable definitions & types
    // ===========================================================================

    #[test]
    fn test_variable_define_int() {
        let ast = parse("let x = 42;");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::VariableAss(v) => {
                assert!(v.new, "new flag");
                assert_eq!(v.name, "x");
                assert_eq!(v.expr, CalcExpr::Value(AstValue::Number(42.0)));
            }
            other => panic!("Expected VariableAss, got {:?}", other),
        }
    }

    #[test]
    fn test_variable_define_string() {
        let ast = parse(r#"let s = "hello";"#);
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::VariableAss(v) => {
                assert!(v.new);
                assert_eq!(v.name, "s");
                assert_eq!(
                    v.expr,
                    CalcExpr::Value(AstValue::String("hello".to_string()))
                );
            }
            other => panic!("Expected VariableAss, got {:?}", other),
        }
    }

    #[test]
    fn test_variable_define_empty_string() {
        let ast = parse(r#"let s = "";"#);
        match &ast.stats[0] {
            DioAstStatement::VariableAss(v) => {
                assert_eq!(v.name, "s");
                assert_eq!(v.expr, CalcExpr::Value(AstValue::String(String::new())));
            }
            other => panic!("Expected VariableAss, got {:?}", other),
        }
    }

    #[test]
    fn test_string_invalid_escape_rejected() {
        let result = DioscriptAst::from_string(r#"let s = "\q";"#);
        assert!(result.is_err(), "unknown escape sequences must be rejected");
    }

    #[test]
    fn test_variable_define_bool() {
        let ast = parse("let b = true;");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::VariableAss(v) => {
                assert!(v.new);
                assert_eq!(v.name, "b");
                assert_eq!(v.expr, CalcExpr::Value(AstValue::Boolean(true)));
            }
            other => panic!("Expected VariableAss, got {:?}", other),
        }
    }

    #[test]
    fn test_variable_reassign() {
        let ast = parse("x = 99;");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::VariableAss(v) => {
                assert!(!v.new, "re-assignment should have new=false");
                assert_eq!(v.name, "x");
                assert_eq!(v.expr, CalcExpr::Value(AstValue::Number(99.0)));
            }
            other => panic!("Expected VariableAss, got {:?}", other),
        }
    }

    // ===========================================================================
    // 2. Arithmetic expressions
    // ===========================================================================

    #[test]
    fn test_add_expression() {
        let ast = parse("1 + 2;");
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => {
                assert_eq!(
                    *expr,
                    CalcExpr::Add(
                        Box::new(CalcExpr::Value(AstValue::Number(1.0))),
                        Box::new(CalcExpr::Value(AstValue::Number(2.0))),
                    )
                );
            }
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_sub_expression() {
        let ast = parse("3 - 4;");
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => {
                assert_eq!(
                    *expr,
                    CalcExpr::Sub(
                        Box::new(CalcExpr::Value(AstValue::Number(3.0))),
                        Box::new(CalcExpr::Value(AstValue::Number(4.0))),
                    )
                );
            }
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_mul_expression() {
        let ast = parse("5 * 6;");
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => {
                assert_eq!(
                    *expr,
                    CalcExpr::Mul(
                        Box::new(CalcExpr::Value(AstValue::Number(5.0))),
                        Box::new(CalcExpr::Value(AstValue::Number(6.0))),
                    )
                );
            }
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_div_expression() {
        let ast = parse("8 / 2;");
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => {
                assert_eq!(
                    *expr,
                    CalcExpr::Div(
                        Box::new(CalcExpr::Value(AstValue::Number(8.0))),
                        Box::new(CalcExpr::Value(AstValue::Number(2.0))),
                    )
                );
            }
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_mod_expression() {
        let ast = parse("7 % 3;");
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => {
                assert_eq!(
                    *expr,
                    CalcExpr::Mod(
                        Box::new(CalcExpr::Value(AstValue::Number(7.0))),
                        Box::new(CalcExpr::Value(AstValue::Number(3.0))),
                    )
                );
            }
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_operator_precedence() {
        let ast = parse("1 + 2 * 3;");
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => {
                // Multiplication should be deeper (higher precedence)
                assert_eq!(
                    *expr,
                    CalcExpr::Add(
                        Box::new(CalcExpr::Value(AstValue::Number(1.0))),
                        Box::new(CalcExpr::Mul(
                            Box::new(CalcExpr::Value(AstValue::Number(2.0))),
                            Box::new(CalcExpr::Value(AstValue::Number(3.0))),
                        )),
                    )
                );
            }
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_comparison_equal() {
        let ast = parse("1 == 2;");
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => {
                assert_eq!(
                    *expr,
                    CalcExpr::Eq(
                        Box::new(CalcExpr::Value(AstValue::Number(1.0))),
                        Box::new(CalcExpr::Value(AstValue::Number(2.0))),
                    )
                );
            }
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    // ===========================================================================
    // 3. Conditional statements (if / if-else)
    // ===========================================================================

    #[test]
    fn test_if_statement() {
        let ast = parse("if true { return 1; }");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::IfStatement(cond) => {
                assert_eq!(cond.condition, CalcExpr::Value(AstValue::Boolean(true)));
                assert_eq!(cond.inner.len(), 1);
                assert!(cond.otherwise.is_none());
            }
            other => panic!("Expected IfStatement, got {:?}", other),
        }
    }

    #[test]
    fn test_if_else_statement() {
        let ast = parse("if true { } else { }");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::IfStatement(cond) => {
                assert_eq!(cond.condition, CalcExpr::Value(AstValue::Boolean(true)));
                assert_eq!(cond.inner.len(), 0);
                let otherwise = cond.otherwise.as_ref().expect("otherwise branch");
                assert_eq!(otherwise.len(), 0);
            }
            other => panic!("Expected IfStatement, got {:?}", other),
        }
    }

    // ===========================================================================
    // 4. Loops (while / for-in)
    // ===========================================================================

    #[test]
    fn test_while_loop() {
        let ast = parse("while true { }");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::LoopStatement(ls) => {
                match &ls.execute_type {
                    LoopExecuteType::Conditional(expr) => {
                        assert_eq!(*expr, CalcExpr::Value(AstValue::Boolean(true)));
                    }
                    other => panic!("Expected Conditional loop, got {:?}", other),
                }
                assert_eq!(ls.inner.len(), 0);
            }
            other => panic!("Expected LoopStatement, got {:?}", other),
        }
    }

    #[test]
    fn test_for_in_loop() {
        let ast = parse("for i in (arr) { }");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::LoopStatement(ls) => match &ls.execute_type {
                LoopExecuteType::Iter { iter, var } => {
                    assert_eq!(var, "i");
                    assert_eq!(
                        *iter,
                        CalcExpr::Value(AstValue::Variable("arr".to_string()))
                    );
                }
                other => panic!("Expected Iter loop, got {:?}", other),
            },
            other => panic!("Expected LoopStatement, got {:?}", other),
        }
    }

    // ===========================================================================
    // 5. Functions
    // ===========================================================================

    #[test]
    fn test_function_define_no_params() {
        let ast = parse("fn foo() { return 42; }");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::FunctionDefine(func) => {
                assert_eq!(func.name, Some("foo".to_string()));
                assert!(func.params.is_empty());
                assert_eq!(func.inner.len(), 1);
            }
            other => panic!("Expected FunctionDefine, got {:?}", other),
        }
    }

    #[test]
    fn test_function_define_with_params() {
        let ast = parse("fn add(a, b) { return a + b; }");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::FunctionDefine(func) => {
                assert_eq!(func.name, Some("add".to_string()));
                assert_eq!(func.params, vec!["a".to_string(), "b".to_string()]);
            }
            other => panic!("Expected FunctionDefine, got {:?}", other),
        }
    }

    #[test]
    fn test_function_duplicate_params_rejected() {
        let result = DioscriptAst::from_string("fn bad(a, a) { return a; }");
        assert!(
            result.is_err(),
            "duplicate function parameters must be rejected"
        );
    }

    #[test]
    fn test_function_duplicate_variadic_param_rejected() {
        let result = DioscriptAst::from_string("fn bad(a, *a) { return a; }");
        assert!(
            result.is_err(),
            "variadic parameter must not duplicate fixed params"
        );
    }

    // ===========================================================================
    // 6. List and Dict literals
    // ===========================================================================

    #[test]
    fn test_list_literal() {
        let ast = parse("let arr = [1, 2, 3];");
        match &ast.stats[0] {
            DioAstStatement::VariableAss(v) => {
                assert!(v.new);
                assert_eq!(v.name, "arr");
                match &v.expr {
                    CalcExpr::Value(AstValue::List(items)) => {
                        assert_eq!(items.len(), 3);
                        assert_eq!(items[0], CalcExpr::Value(AstValue::Number(1.0)));
                        assert_eq!(items[1], CalcExpr::Value(AstValue::Number(2.0)));
                        assert_eq!(items[2], CalcExpr::Value(AstValue::Number(3.0)));
                    }
                    other => panic!("Expected List value, got {:?}", other),
                }
            }
            other => panic!("Expected VariableAss, got {:?}", other),
        }
    }

    #[test]
    fn test_dict_literal() {
        let ast = parse(r#"let obj = { "a": 1, "b": "two" };"#);
        match &ast.stats[0] {
            DioAstStatement::VariableAss(v) => {
                assert!(v.new);
                assert_eq!(v.name, "obj");
                match &v.expr {
                    CalcExpr::Value(AstValue::Dict(map)) => {
                        assert_eq!(map.len(), 2);
                        assert_eq!(map.get("a"), Some(&CalcExpr::Value(AstValue::Number(1.0))));
                        assert_eq!(
                            map.get("b"),
                            Some(&CalcExpr::Value(AstValue::String("two".to_string())))
                        );
                    }
                    other => panic!("Expected Dict value, got {:?}", other),
                }
            }
            other => panic!("Expected VariableAss, got {:?}", other),
        }
    }

    // ===========================================================================
    // 7. Elements (JSX-like)
    // ===========================================================================

    #[test]
    fn test_element_simple() {
        let ast = parse(r#"div { "hello" };"#);
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => match expr {
                CalcExpr::Value(AstValue::Element(el)) => {
                    assert_eq!(el.name, "div");
                    assert!(el.attributes.is_empty());
                    assert_eq!(el.content.len(), 1);
                    match &el.content[0] {
                        AstElementContentType::InlineExpr(expr) => {
                            assert_eq!(
                                *expr,
                                CalcExpr::Value(AstValue::String("hello".to_string()))
                            );
                        }
                        other => panic!("Expected InlineExpr, got {:?}", other),
                    }
                }
                other => panic!("Expected Element value, got {:?}", other),
            },
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_element_with_attrs_and_child() {
        let ast = parse(r#"div { class: "main", p { "child" } };"#);
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => match expr {
                CalcExpr::Value(AstValue::Element(el)) => {
                    assert_eq!(el.name, "div");
                    // attribute
                    assert_eq!(el.attributes.len(), 1);
                    assert_eq!(
                        el.attributes.get("class"),
                        Some(&AstValue::String("main".to_string()))
                    );
                    // child element
                    assert_eq!(el.content.len(), 1);
                    match &el.content[0] {
                        AstElementContentType::Children(child) => {
                            assert_eq!(child.name, "p");
                            assert_eq!(child.content.len(), 1);
                            match &child.content[0] {
                                AstElementContentType::InlineExpr(expr) => {
                                    assert_eq!(
                                        *expr,
                                        CalcExpr::Value(AstValue::String("child".to_string()))
                                    );
                                }
                                other => panic!("Expected InlineExpr, got {:?}", other),
                            }
                        }
                        other => panic!("Expected Children, got {:?}", other),
                    }
                }
                other => panic!("Expected Element value, got {:?}", other),
            },
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_element_duplicate_attrs_rejected() {
        let result = DioscriptAst::from_string(r#"div { class: "a", class: "b" };"#);
        assert!(
            result.is_err(),
            "duplicate element attributes must be rejected"
        );
    }

    // ===========================================================================
    // 8. Comments
    // ===========================================================================

    #[test]
    fn test_line_comment() {
        let ast = parse("// this is a comment");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::LineComment(s) => {
                assert_eq!(s, "this is a comment");
            }
            other => panic!("Expected LineComment, got {:?}", other),
        }
    }

    // ===========================================================================
    // 9. Module use
    // ===========================================================================

    #[test]
    fn test_module_use() {
        let ast = parse("use std::io;");
        assert_eq!(ast.stats.len(), 1);
        match &ast.stats[0] {
            DioAstStatement::ModuleUse(use_stmt) => {
                assert_eq!(use_stmt.0, vec!["std".to_string(), "io".to_string()]);
            }
            other => panic!("Expected ModuleUse, got {:?}", other),
        }
    }

    #[test]
    fn test_module_use_deep() {
        let ast = parse("use std::collections::hash_map;");
        match &ast.stats[0] {
            DioAstStatement::ModuleUse(use_stmt) => {
                assert_eq!(
                    use_stmt.0,
                    vec![
                        "std".to_string(),
                        "collections".to_string(),
                        "hash_map".to_string()
                    ]
                );
            }
            other => panic!("Expected ModuleUse, got {:?}", other),
        }
    }

    // ===========================================================================
    // 10. Multiple statements / integration
    // ===========================================================================

    #[test]
    fn test_multiple_statements() {
        let ast = parse("let x = 1;\nlet y = 2;\nx + y;");
        assert_eq!(ast.stats.len(), 3);
        // 1st: variable
        match &ast.stats[0] {
            DioAstStatement::VariableAss(v) => {
                assert_eq!(v.name, "x");
            }
            other => panic!("Expected VariableAss, got {:?}", other),
        }
        // 2nd: variable
        match &ast.stats[1] {
            DioAstStatement::VariableAss(v) => {
                assert_eq!(v.name, "y");
            }
            other => panic!("Expected VariableAss, got {:?}", other),
        }
        // 3rd: expression
        match &ast.stats[2] {
            DioAstStatement::CalcExpr(_) => {}
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_function_call() {
        let ast = parse("print(42);");
        match &ast.stats[0] {
            DioAstStatement::FunctionCall(call) => {
                assert_eq!(call.name, FunctionName::Single("print".to_string()));
                assert_eq!(call.arguments.len(), 1);
                assert_eq!(call.arguments[0], CalcExpr::Value(AstValue::Number(42.0)));
            }
            other => panic!("Expected FunctionCall, got {:?}", other),
        }
    }

    #[test]
    fn test_return_statement() {
        let ast = parse("return 42;");
        match &ast.stats[0] {
            DioAstStatement::ReturnValue(expr) => {
                assert_eq!(*expr, CalcExpr::Value(AstValue::Number(42.0)));
            }
            other => panic!("Expected ReturnValue, got {:?}", other),
        }
    }

    #[test]
    fn test_variable_expression() {
        let ast = parse("x;");
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => {
                assert_eq!(*expr, CalcExpr::Value(AstValue::Variable("x".to_string())));
            }
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_nested_arithmetic() {
        let ast = parse("(1 + 2) * 3;");
        match &ast.stats[0] {
            DioAstStatement::CalcExpr(expr) => {
                assert_eq!(
                    *expr,
                    CalcExpr::Mul(
                        Box::new(CalcExpr::Add(
                            Box::new(CalcExpr::Value(AstValue::Number(1.0))),
                            Box::new(CalcExpr::Value(AstValue::Number(2.0))),
                        )),
                        Box::new(CalcExpr::Value(AstValue::Number(3.0))),
                    )
                );
            }
            other => panic!("Expected CalcExpr, got {:?}", other),
        }
    }

    // ===========================================================================
    // 11. Error handling
    // ===========================================================================

    #[test]
    fn test_parse_empty_input() {
        let result = DioscriptAst::from_string("");
        assert!(result.is_ok());
        assert!(result.unwrap().stats.is_empty());
    }

    #[test]
    fn test_parse_error() {
        let result = DioscriptAst::from_string("let x = ;");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unmatched_trailing() {
        let result = DioscriptAst::from_string("let x = 1; garbage");
        assert!(result.is_err());
    }
}
