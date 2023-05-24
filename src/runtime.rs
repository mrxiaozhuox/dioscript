use std::collections::HashMap;

use id_tree::{Node, NodeId, Tree, TreeBuilder};

use crate::{
    ast::{CalculateMark, DioAstStatement, DioscriptAst, SubExpr},
    element::Element,
    error::RuntimeError,
    types::Value,
};

pub struct Runtime {
    // reference content: use for save reference node-id.
    refs: HashMap<String, NodeId>,
    // scope tree: use for build scope structure.
    scope: Tree<ScopeType>,
    // root scope: root tree id.
    root_scope: NodeId,
}

impl Runtime {
    pub fn new() -> Self {
        let mut scope = TreeBuilder::new().build();
        let root = scope
            .insert(Node::new(ScopeType::Block), id_tree::InsertBehavior::AsRoot)
            .expect("Scope init failed.");
        Self {
            refs: HashMap::new(),
            scope,
            root_scope: root,
        }
    }

    pub fn execute_ast(&mut self, ast: DioscriptAst) -> Result<Value, RuntimeError> {
        let root_id = self.root_scope.clone();
        let result = self.execute_scope(ast.stats, &root_id)?;
        Ok(result)
    }

    pub fn execute_scope(
        &mut self,
        statements: Vec<DioAstStatement>,
        current_scope: &NodeId,
    ) -> Result<Value, RuntimeError> {
        let mut result: Value = Value::None;
        let mut finish = false;
        for v in statements {
            if finish {
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
                    result = self.execute_calculate(r.clone(), current_scope)?;
                    finish = true;
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
                            result = self.execute_scope(inner_ast, &sub_scope)?;
                            finish = true;
                        } else {
                            if let Some(otherwise) = otherwise {
                                result = self.execute_scope(otherwise, &sub_scope)?;
                                finish = true;
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
        if let Value::Element(e) = result {
            result = Value::Element(self.execute_element(e, current_scope)?);
        }
        return Ok(result);
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
        if let Some(ref_scope) = self.refs.get(name) {
            if let Ok(ref_node) = self.scope.get(ref_scope) {
                let mut flag = false;
                let ref_node_id = ref_node.parent().unwrap();
                let mut curr_node_id = current_scope;

                loop {
                    if curr_node_id == ref_node_id {
                        flag = true;
                        break;
                    }
                    if let Some(v) = self.scope.get(curr_node_id)?.parent() {
                        curr_node_id = v;
                    } else {
                        break;
                    }
                }
                if flag {
                    if let ScopeType::Reference(v) = ref_node.data() {
                        println!("{v:?}");
                        return Ok((ref_node_id.clone(), v.clone()));
                    }
                }
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
        let mut value = value;

        // handle element data type
        if let Value::Element(element) = &value {
            value = Value::Element(self.execute_element(element.clone(), current_scope)?);
        }

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

    fn execute_element(
        &mut self,
        element: Element,
        current_scope: &NodeId,
    ) -> Result<Element, RuntimeError> {
        let mut attrs = HashMap::new();
        for i in element.attributes {
            let name = i.0;
            let data = i.1;
            if let Value::Reference(r) = data {
                let result = self.get_ref(&r, current_scope)?;
                attrs.insert(name, result.1.value);
            } else {
                attrs.insert(name, data);
            }
        }
        let mut content = vec![];
        for i in element.content {
            match i {
                crate::element::ElementContentType::Children(v) => {
                    let executed_element = self.execute_element(v, current_scope)?;
                    content.push(crate::element::ElementContentType::Children(
                        executed_element,
                    ));
                }
                crate::element::ElementContentType::Content(v) => {
                    content.push(crate::element::ElementContentType::Content(v));
                }
                crate::element::ElementContentType::Condition(v) => {
                    let value = self.execute_calculate(v.condition.0.clone(), current_scope)?;
                    let sub_scope = self.scope.insert(
                        Node::new(ScopeType::Block),
                        id_tree::InsertBehavior::UnderNode(current_scope),
                    )?;
                    if let Value::Boolean(b) = value {
                        let mut temp = Value::None;
                        if b {
                            temp = self.execute_scope(v.inner, &sub_scope)?;
                        } else {
                            if let Some(otherwise) = v.otherwise {
                                temp = self.execute_scope(otherwise, &sub_scope)?;
                            }
                        }
                        println!("{:?}", temp);
                        if let Value::Tuple((k, v)) = &temp {
                            if let Value::String(k) = *k.clone() {
                                attrs.insert(k.to_string(), *v.clone());
                            }
                        }
                        if let Value::String(v) = &temp {
                            content.push(crate::element::ElementContentType::Content(v.clone()));
                        }
                        if let Value::Element(v) = temp {
                            content.push(crate::element::ElementContentType::Children(v));
                        }
                    }
                }
                crate::element::ElementContentType::Reference(v) => {
                    let result = self.get_ref(&v, current_scope)?;
                    if let Value::String(s) = result.1.value {
                        content.push(crate::element::ElementContentType::Content(s));
                    }
                }
            }
        }
        Ok(Element {
            name: element.name,
            attributes: attrs,
            content,
        })
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
