use std::collections::HashMap;

use id_tree::{Node, NodeId, Tree, TreeBuilder};

use crate::{
    ast::{CalculateMark, DioAstStatement, DioscriptAst, SubExpr},
    element::AstElement,
    error::RuntimeError,
    types::{AstValue, Value},
};

pub struct Runtime {
    // variable content: use for save variable node-id.
    vars: HashMap<String, NodeId>,
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
            vars: HashMap::new(),
            scope,
            root_scope: root,
        }
    }

    pub fn execute_ast(&mut self, ast: DioscriptAst) -> Result<Value, RuntimeError> {
        let root_id = self.root_scope.clone();
        let result = self.execute_scope(ast.stats, &root_id)?;
        Ok(Value::from_ast_value(result))
    }

    pub fn execute_scope(
        &mut self,
        statements: Vec<DioAstStatement>,
        current_scope: &NodeId,
    ) -> Result<AstValue, RuntimeError> {
        let mut result: AstValue = AstValue::None;
        let mut finish = false;
        for v in statements {
            if finish {
                break;
            }
            match v {
                crate::ast::DioAstStatement::VariableAss(var) => {
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
                    if let AstValue::Boolean(state) = state {
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
        if let AstValue::Element(e) = result {
            result = AstValue::Element(self.execute_element(e, current_scope)?);
        }
        return Ok(result);
    }

    fn execute_calculate(
        &self,
        expr: Vec<(CalculateMark, SubExpr)>,
        current_scope: &NodeId,
    ) -> Result<AstValue, RuntimeError> {
        let mut buf_value = AstValue::None;
        for pair in expr {
            let signal = pair.0;
            let info = pair.1;
            let content = match info {
                SubExpr::Single(info) => {
                    let mut content = info.1;

                    if let AstValue::Variable(r) = &content {
                        let (_, data) = self.get_ref(r, current_scope)?;
                        content = data.value.clone();
                    }

                    if info.0 {
                        if let AstValue::Boolean(b) = content {
                            content = AstValue::Boolean(!b);
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
    ) -> Result<(NodeId, Variable), RuntimeError> {
        if let Some(ref_scope) = self.vars.get(name) {
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
                    if let ScopeType::Variable(v) = ref_node.data() {
                        println!("{v:?}");
                        return Ok((ref_node_id.clone(), v.clone()));
                    }
                }
            }
        }
        Err(RuntimeError::VariableNotFound {
            name: name.to_string(),
        })
    }

    fn set_ref(
        &mut self,
        name: &str,
        value: AstValue,
        current_scope: &NodeId,
    ) -> Result<NodeId, RuntimeError> {
        let mut value = value;

        // handle element data type
        if let AstValue::Element(element) = &value {
            value = AstValue::Element(self.execute_element(element.clone(), current_scope)?);
        }

        if let Some(scope) = self.vars.get(name) {
            let mut vars = self.scope.get_mut(scope)?.data_mut().as_variable().unwrap();
            // change variable value
            vars.value = value;
            vars.counter += 1;
            return Ok(scope.clone());
        } else {
            let new_scope = self.scope.insert(
                Node::new(ScopeType::Variable(Variable { value, counter: 1 })),
                id_tree::InsertBehavior::UnderNode(current_scope),
            )?;
            self.vars.insert(name.to_string(), new_scope.clone());
            return Ok(new_scope);
        }
    }

    fn execute_element(
        &mut self,
        element: AstElement,
        current_scope: &NodeId,
    ) -> Result<AstElement, RuntimeError> {
        let mut attrs = HashMap::new();
        for i in element.attributes {
            let name = i.0;
            let data = i.1;
            if let AstValue::Variable(r) = data {
                let result = self.get_ref(&r, current_scope)?;
                attrs.insert(name, result.1.value);
            } else {
                attrs.insert(name, data);
            }
        }
        let mut content = vec![];
        for i in element.content {
            match i {
                crate::element::AstElementContentType::Children(v) => {
                    let executed_element = self.execute_element(v, current_scope)?;
                    content.push(crate::element::AstElementContentType::Children(
                        executed_element,
                    ));
                }
                crate::element::AstElementContentType::Content(v) => {
                    content.push(crate::element::AstElementContentType::Content(v));
                }
                crate::element::AstElementContentType::Condition(v) => {
                    let value = self.execute_calculate(v.condition.0.clone(), current_scope)?;
                    let sub_scope = self.scope.insert(
                        Node::new(ScopeType::Block),
                        id_tree::InsertBehavior::UnderNode(current_scope),
                    )?;
                    if let AstValue::Boolean(b) = value {
                        let mut temp = AstValue::None;
                        if b {
                            temp = self.execute_scope(v.inner, &sub_scope)?;
                        } else {
                            if let Some(otherwise) = v.otherwise {
                                temp = self.execute_scope(otherwise, &sub_scope)?;
                            }
                        }
                        println!("{:?}", temp);
                        if let AstValue::Tuple((k, v)) = &temp {
                            if let AstValue::String(k) = *k.clone() {
                                attrs.insert(k.to_string(), *v.clone());
                            }
                        }
                        if let AstValue::String(v) = &temp {
                            content.push(crate::element::AstElementContentType::Content(v.clone()));
                        }
                        if let AstValue::Element(v) = temp {
                            content.push(crate::element::AstElementContentType::Children(v));
                        }
                    }
                }
                crate::element::AstElementContentType::Variable(v) => {
                    let result = self.get_ref(&v, current_scope)?;
                    if let AstValue::String(s) = result.1.value {
                        content.push(crate::element::AstElementContentType::Content(s));
                    }
                }
            }
        }
        Ok(AstElement {
            name: element.name,
            attributes: attrs,
            content,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ScopeType {
    Block,
    Variable(Variable),
}

impl ScopeType {
    pub fn as_variable(&self) -> Option<Variable> {
        if let Self::Variable(r) = self {
            return Some(r.clone());
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub value: AstValue,
    pub counter: u32,
}
