use std::collections::HashMap;

use id_tree::{Node, NodeId, Tree, TreeBuilder};

use crate::{
    ast::{CalculateMark, DioAstStatement, DioscriptAst, SubExpr},
    error::RuntimeError,
    parser::CalcExpr,
    types::Value,
};

pub struct Runtime {
    // reference content: use for save reference node-id.
    refs: HashMap<String, NodeId>,
    // scope tree: use for build scope structure.
    scope: Tree<ScopeType>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
            scope: TreeBuilder::new().build(),
        }
    }

    pub fn execute_ast(&mut self, ast: DioscriptAst) -> Result<Value, RuntimeError> {
        let root = self
            .scope
            .insert(Node::new(ScopeType::Block), id_tree::InsertBehavior::AsRoot)
            .expect("Scope init failed.");
        let result = self.execute_scope(ast.stats, &root)?;
        Ok(result)
    }

    pub fn execute_scope(
        &mut self,
        statements: Vec<DioAstStatement>,
        current_scope: &NodeId,
    ) -> Result<Value, RuntimeError> {
        let mut result: Option<CalcExpr> = None;
        for v in statements {
            if result.is_some() {
                break;
            }
            match v {
                crate::ast::DioAstStatement::ReferenceAss(var) => {
                    let name = var.0.clone();
                    let value = var.1.clone();
                    let value = self.execute_calculate(value, current_scope)?;
                    let _scope = self.set_ref(&name, value, current_scope)?;
                }
                crate::ast::DioAstStatement::ReturnValue(r) => {
                    result = Some(r.clone());
                }
                crate::ast::DioAstStatement::IfStatement(cond) => {
                    let sub_scope = self.scope.insert(
                        Node::new(ScopeType::Block),
                        id_tree::InsertBehavior::UnderNode(current_scope),
                    )?;

                    let condition_expr = cond.condition.0.clone();
                    let inner_ast = cond.inner.clone();
                    let otherwise = cond.otherwise.clone();
                    let state = self.execute_calculate(condition_expr, current_scope)?;
                    if let Value::Boolean(state) = state {
                        if state {
                            let temp = self.execute_scope(inner_ast, &sub_scope)?;
                            result = Some(temp.to_calc_expr());
                        } else {
                            if let Some(otherwise) = otherwise {
                                let temp = self.execute_scope(otherwise, &sub_scope)?;
                                result = Some(temp.to_calc_expr());
                            }
                        }
                    } else {
                        return Err(RuntimeError::IllegalTypeInConditional {
                            value_type: state.value_name(),
                        });
                    }
                }
            }
        }
        if let Some(result) = result {
            let result = self.execute_calculate(result, current_scope)?;
            Ok(result)
        } else {
            Ok(Value::None)
        }
    }

    fn execute_calculate(
        &self,
        expr: Vec<(CalculateMark, SubExpr)>,
        current_scope: &NodeId,
    ) -> Result<Value, RuntimeError> {
        let mut buf_value = Value::None;
        for pair in expr {
            let signal = pair.0;
            let info = pair.1;
            let content = match info {
                SubExpr::Single(info) => {
                    let mut content = info.1;

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
                    let v = self.execute_calculate(p.0, current_scope)?;
                    v
                }
            };

            if signal.to_string() != "none".to_string() {
                let matched_value = buf_value.calc(&content, signal)?;
                buf_value = matched_value;
            } else {
                buf_value = content;
            }
        }

        Ok(buf_value)
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
                    return Ok((scope.clone(), node.data().clone().as_reference().unwrap()));
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
            let mut refs = self
                .scope
                .get_mut(scope)?
                .data_mut()
                .as_reference()
                .unwrap();
            // change reference value
            refs.value = value;
            refs.counter += 1;
            return Ok(scope.clone());
        } else {
            let new_scope = self.scope.insert(
                Node::new(ScopeType::Reference(Reference { value, counter: 1 })),
                id_tree::InsertBehavior::UnderNode(current_scope),
            )?;
            self.refs.insert(name.to_string(), new_scope.clone());
            return Ok(new_scope);
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScopeType {
    Block,
    Reference(Reference),
}

impl ScopeType {
    pub fn as_reference(&self) -> Option<Reference> {
        if let Self::Reference(r) = self {
            return Some(r.clone());
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub value: Value,
    pub counter: u32,
}
