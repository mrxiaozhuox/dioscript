use std::collections::HashMap;

use crate::{
    ast::{ConditionalSignal, DioAstStatement, DioscriptAst, SubExpr},
    error::{Error, RuntimeError},
    types::Value,
};

pub struct Runtime {
    // reference content
    refs: HashMap<String, Reference>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
        }
    }

    pub fn execute_ast(ast: DioscriptAst) -> Result<Value, RuntimeError> {
        Ok(Value::None)
    }

    pub fn execute_scope(
        &mut self,
        statements: Vec<DioAstStatement>,
        scope_id: u32,
    ) -> Result<Value, RuntimeError> {
        let mut result = Value::None;
        for v in statements {
            match v {
                crate::ast::DioAstStatement::ReferenceAss(var) => {
                    let name = var.0.clone();
                    let value = var.1.clone();
                    if let Some(mut origin) = self.refs.get_mut(&name) {
                        origin.value = value;
                        origin.counter += 1;
                        origin.latest_change = scope_id;
                    } else {
                        self.refs.insert(
                            name,
                            Reference {
                                value,
                                scope: scope_id,
                                counter: 1,
                                latest_change: scope_id,
                            },
                        );
                    }
                }
                crate::ast::DioAstStatement::ReturnValue(r) => {
                    result = r.clone();
                }
                crate::ast::DioAstStatement::IfStatement(cond) => {
                    let condition_expr = cond.condition.0.clone();
                    let inner_ast = cond.inner.clone();
                    let otherwise = cond.otherwise.clone();
                }
            }
        }
        Ok(result)
    }

    fn verify_condition(expr: Vec<(ConditionalSignal, SubExpr)>) -> Result<bool, RuntimeError> {
        let mut current_state = false;
        let mut buf_value = Value::None;
        for pair in expr {
            let signal = pair.0;
            let info = pair.1;
            if let SubExpr::Single(info) = info {
                let mut content = info.1;
                // handle ! signal
                if info.0 {
                    if let Value::Boolean(b) = content {
                        content = Value::Boolean(!b);
                    } else {
                        return Err(RuntimeError::illegal_operator_for_type(
                            &signal.to_string(),
                            &content.value_name(),
                        ));
                    }
                }
            }
        }
        Ok(false)
    }
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub value: Value,
    pub scope: u32,
    pub counter: u32,
    pub latest_change: u32,
}
