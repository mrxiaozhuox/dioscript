use std::collections::HashMap;

use id_tree::{Node, NodeId, Tree, TreeBuilder};

use crate::{
    ast::{ConditionalSignal, DioAstStatement, DioscriptAst, SubExpr},
    error::RuntimeError,
    types::Value,
};

pub struct Runtime {
    // reference content
    refs: HashMap<String, NodeId>,
    scope: Tree<Reference>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
            scope: TreeBuilder::new().build(),
        }
    }

    pub fn execute_ast(&mut self, ast: DioscriptAst) -> Result<Value, RuntimeError> {
        let id = self
            .scope
            .insert(
                Node::new(Reference {
                    value: Value::Boolean(true),
                    counter: 0,
                }),
                id_tree::InsertBehavior::AsRoot,
            )
            .expect("Scope init failed.");
        let result = self.execute_scope(ast.stats, &id)?;
        Ok(result)
    }

    pub fn execute_scope(
        &mut self,
        statements: Vec<DioAstStatement>,
        current_scope: &NodeId,
    ) -> Result<Value, RuntimeError> {
        let mut result = Value::None;
        let mut return_state = false;
        for v in statements {
            if return_state {
                break;
            }
            match v {
                crate::ast::DioAstStatement::ReferenceAss(var) => {
                    let name = var.0.clone();
                    let value = var.1.clone();
                    let _scope = self.set_ref(&name, value, current_scope)?;
                }
                crate::ast::DioAstStatement::ReturnValue(r) => {
                    result = r.clone();
                    return_state = true;
                }
                crate::ast::DioAstStatement::IfStatement(cond) => {
                    let condition_expr = cond.condition.0.clone();
                    let inner_ast = cond.inner.clone();
                    let otherwise = cond.otherwise.clone();
                    let state = self.verify_condition(condition_expr, current_scope)?;
                    if state {
                        result = self.execute_scope(inner_ast, current_scope)?;
                        return_state = true;
                    } else {
                        if let Some(otherwise) = otherwise {
                            result = self.execute_scope(otherwise, current_scope)?;
                            return_state = true;
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    fn verify_condition(
        &self,
        expr: Vec<(ConditionalSignal, SubExpr)>,
        current_scope: &NodeId,
    ) -> Result<bool, RuntimeError> {
        let mut buf_value = Value::None;
        for pair in expr {
            let signal = pair.0;
            let info = pair.1;
            let content = match info {
                SubExpr::Single(info) => {
                    let mut content = info.1;
                    // handle ! signal

                    if let Value::Reference(r) = &content {
                        let (_, data) = self.get_ref(r, current_scope)?;
                        content = data.value.clone();
                    }

                    if info.0 {
                        if let Value::Boolean(b) = content {
                            content = Value::Boolean(!b);
                        } else {
                            return Err(RuntimeError::IllegalOperatorForType {
                                operator: signal.to_string(),
                                value_type: content.value_name(),
                            });
                        }
                    }
                    content
                }
                SubExpr::Pair(p) => {
                    let v = self.verify_condition(p.0, current_scope)?;
                    Value::Boolean(v)
                }
            };

            println!("{:?}", signal);
            if signal.to_string() != "".to_string() {
                let matched_value = buf_value.compare(&content, signal)?;
                buf_value = Value::Boolean(matched_value);
            } else {
                buf_value = content;
            }
        }

        if let Value::Boolean(v) = buf_value {
            return Ok(v);
        }

        Ok(false)
    }

    fn get_ref(
        &self,
        name: &str,
        current_scope: &NodeId,
    ) -> Result<(NodeId, Reference), RuntimeError> {
        if let Some(scope) = self.refs.get(name) {
            let data = self.scope.get(scope);
            if let Ok(node) = data {
                // loop to found all parent.
                let mut parent = node;
                let mut flag = false;
                while let Some(curr) = parent.parent() {
                    if curr == current_scope {
                        flag = true;
                        break;
                    }
                    parent = self.scope.get(curr).unwrap();
                }

                if flag {
                    return Ok((scope.clone(), node.data().clone()));
                } else {
                    return Err(RuntimeError::ReferenceNotFound {
                        name: name.to_string(),
                    });
                }
            } else {
                return Err(RuntimeError::ReferenceNotFound {
                    name: name.to_string(),
                });
            }
        }
        Err(RuntimeError::ReferenceNotFound {
            name: name.to_string(),
        })
    }

    fn set_ref(
        &mut self,
        name: &str,
        value: Value,
        current_scope: &NodeId,
    ) -> Result<NodeId, RuntimeError> {
        if let Some(scope) = self.refs.get(name) {
            let mut refs = self.scope.get_mut(scope)?.data_mut();
            refs.value = value;
            refs.counter += 1;
            return Ok(scope.clone());
        } else {
            let new_scope = self.scope.insert(
                Node::new(Reference { value, counter: 1 }),
                id_tree::InsertBehavior::UnderNode(current_scope),
            )?;
            self.refs.insert(name.to_string(), new_scope.clone());
            return Ok(new_scope);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub value: Value,
    pub counter: u32,
}
