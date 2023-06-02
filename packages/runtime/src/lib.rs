use std::collections::HashMap;

use id_tree::{Node, NodeId, Tree, TreeBuilder};

use dioscript_parser::{
    ast::{CalculateMark, DioAstStatement, DioscriptAst, LoopExecuteType},
    element::{AstElement, AstElementContentType},
    error::{Error, RuntimeError},
    parser::CalcExpr,
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

    pub fn execute(&mut self, code: &str) -> Result<Value, Error> {
        let ast = DioscriptAst::from_string(code)?;
        Ok(self.execute_ast(ast)?)
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
                DioAstStatement::VariableAss(var) => {
                    let name = var.0.clone();
                    let value = var.1.clone();
                    let value = self.execute_calculate(value, current_scope)?;
                    let _scope = self.set_ref(&name, value, current_scope)?;
                }
                DioAstStatement::ReturnValue(r) => {
                    result = self.execute_calculate(r.clone(), current_scope)?;
                    finish = true;
                }
                DioAstStatement::IfStatement(cond) => {
                    let sub_scope = self.scope.insert(
                        Node::new(ScopeType::Block),
                        id_tree::InsertBehavior::UnderNode(current_scope),
                    )?;

                    let condition_expr = cond.condition.clone();
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
                DioAstStatement::LoopStatement(data) => {
                    let sub_scope = self.scope.insert(
                        Node::new(ScopeType::Block),
                        id_tree::InsertBehavior::UnderNode(current_scope),
                    )?;
                    let execute_type = data.execute_type;
                    match execute_type {
                        LoopExecuteType::Conditional(cond) => loop {
                            let cond = cond.clone();
                            let state = self.execute_calculate(cond, &current_scope)?;
                            let state = state.to_boolean_data();
                            if !state {
                                break;
                            } else {
                                let res = self.execute_scope(data.inner.clone(), &sub_scope)?;
                                if !res.as_none() {
                                    result = res;
                                    finish = true;
                                    break;
                                }
                            }
                        },
                        LoopExecuteType::Iter { mut iter, var } => {
                            if iter.value_name() == "variable" {
                                iter = self
                                    .get_ref(&iter.as_variable().unwrap(), current_scope)?
                                    .1
                                    .value;
                            }
                            if iter.value_name() == "list" {
                                for i in iter.as_list().unwrap() {
                                    self.set_ref(&var, i.clone(), &sub_scope)?;
                                    self.execute_scope(data.inner.clone(), &sub_scope)?;
                                }
                            }
                        }
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
        expr: CalcExpr,
        current_scope: &NodeId,
    ) -> Result<AstValue, RuntimeError> {
        match expr {
            CalcExpr::Value(v) => {
                if let AstValue::Variable(v) = v {
                    let data = self.get_ref(&v, current_scope)?;
                    Ok(data.1.value.clone())
                } else if let AstValue::VariableIndex((name, index)) = v {
                    let temp = self.get_ref(&name, current_scope)?;
                    let data = self.get_from_index(temp.1.value, *index.clone())?;
                    Ok(data)
                } else {
                    Ok(v)
                }
            }
            CalcExpr::Add(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::Plus)
            }
            CalcExpr::Sub(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::Minus)
            }
            CalcExpr::Mul(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::Multiply)
            }
            CalcExpr::Div(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::Divide)
            }
            CalcExpr::Mod(_, _) => Ok(AstValue::Boolean(false)),
            CalcExpr::Eq(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::Equal)
            }
            CalcExpr::Ne(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::NotEqual)
            }
            CalcExpr::Gt(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::Large)
            }
            CalcExpr::Lt(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::Small)
            }
            CalcExpr::Ge(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::LargeOrEqual)
            }
            CalcExpr::Le(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::SmallOrEqual)
            }
            CalcExpr::And(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::And)
            }
            CalcExpr::Or(l, r) => {
                let l = self.execute_calculate(*l, current_scope)?;
                let r = self.execute_calculate(*r, current_scope)?;
                l.calc(&r, CalculateMark::Or)
            }
        }
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
            let vars = self.scope.get_mut(scope)?.data_mut();
            if let ScopeType::Variable(v) = vars {
                v.value = value;
                v.counter += 1;
            }
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

    fn get_from_index(&self, value: AstValue, index: AstValue) -> Result<AstValue, RuntimeError> {
        match &value {
            AstValue::String(v) => {
                if let AstValue::Number(num) = index {
                    let num = num as usize;
                    let chars = v.chars();
                    let c = chars.collect::<Vec<char>>();
                    if c.len() >= num + 1 {
                        return Ok(AstValue::String(c[num].to_string()));
                    } else {
                        Err(RuntimeError::IndexNotFound {
                            index: index.value_name(),
                            value: value.value_name(),
                        })
                    }
                } else {
                    Err(RuntimeError::IllegalIndexType {
                        index_type: index.value_name(),
                        value_type: value.value_name(),
                    })
                }
            }
            AstValue::List(v) => {
                if let AstValue::Number(num) = index {
                    let num = num as usize;
                    if v.len() >= num + 1 {
                        let v = v[num].clone();
                        Ok(v)
                    } else {
                        Err(RuntimeError::IndexNotFound {
                            index: index.value_name(),
                            value: value.value_name(),
                        })
                    }
                } else {
                    Err(RuntimeError::IllegalIndexType {
                        index_type: index.value_name(),
                        value_type: value.value_name(),
                    })
                }
            }
            AstValue::Dict(v) => {
                if let AstValue::String(key) = &index {
                    if let Some(value) = v.get(key) {
                        Ok(value.clone())
                    } else {
                        Err(RuntimeError::IndexNotFound {
                            index: index.value_name(),
                            value: value.value_name(),
                        })
                    }
                } else {
                    Err(RuntimeError::IllegalIndexType {
                        index_type: index.value_name(),
                        value_type: value.value_name(),
                    })
                }
            }
            AstValue::Tuple(v) => {
                if let AstValue::Number(num) = index {
                    let num = num as usize;
                    if num == 0 {
                        Ok(*v.0.clone())
                    } else if num == 1 {
                        Ok(*v.1.clone())
                    } else {
                        Err(RuntimeError::IndexNotFound {
                            index: index.value_name(),
                            value: value.value_name(),
                        })
                    }
                } else {
                    Err(RuntimeError::IllegalIndexType {
                        index_type: index.value_name(),
                        value_type: value.value_name(),
                    })
                }
            }
            _ => Err(RuntimeError::IllegalIndexType {
                index_type: index.value_name(),
                value_type: value.value_name(),
            }),
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
                AstElementContentType::Children(v) => {
                    let executed_element = self.execute_element(v, current_scope)?;
                    content.push(AstElementContentType::Children(
                        executed_element,
                    ));
                }
                AstElementContentType::Content(v) => {
                    content.push(AstElementContentType::Content(v));
                }
                AstElementContentType::Condition(v) => {
                    let value = self.execute_calculate(v.condition, current_scope)?;
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
                        if let AstValue::Tuple((k, v)) = &temp {
                            if let AstValue::String(k) = *k.clone() {
                                attrs.insert(k.to_string(), *v.clone());
                            }
                        }
                        if let AstValue::String(v) = &temp {
                            content.push(AstElementContentType::Content(v.clone()));
                        }
                        if let AstValue::Number(v) = &temp {
                            content.push(AstElementContentType::Content(format!(
                                "{v}"
                            )));
                        }
                        if let AstValue::Element(v) = temp {
                            content.push(AstElementContentType::Children(v));
                        }
                    }
                }
                AstElementContentType::Loop(v) => {
                    let sub_scope = self.scope.insert(
                        Node::new(ScopeType::Block),
                        id_tree::InsertBehavior::UnderNode(current_scope),
                    )?;
                    let execute_type = v.execute_type;
                    match execute_type {
                        LoopExecuteType::Conditional(cond) => loop {
                            let cond = cond.clone();
                            let state = self.execute_calculate(cond, &current_scope)?;
                            let state = state.to_boolean_data();
                            if !state {
                                break;
                            } else {
                                let temp = self.execute_scope(v.inner.clone(), &sub_scope)?;
                                if let AstValue::Tuple((k, v)) = &temp {
                                    if let AstValue::String(k) = *k.clone() {
                                        attrs.insert(k.to_string(), *v.clone());
                                    }
                                }
                                if let AstValue::String(v) = &temp {
                                    content.push(AstElementContentType::Content(
                                        v.clone(),
                                    ));
                                }
                                if let AstValue::Number(v) = &temp {
                                    content.push(AstElementContentType::Content(
                                        format!("{v}"),
                                    ));
                                }
                                if let AstValue::Element(v) = temp {
                                    content
                                        .push(AstElementContentType::Children(v));
                                }
                            }
                        },
                        LoopExecuteType::Iter { mut iter, var } => {
                            if iter.value_name() == "variable" {
                                iter = self
                                    .get_ref(&iter.as_variable().unwrap(), current_scope)?
                                    .1
                                    .value;
                            }
                            if iter.value_name() == "list" {
                                for i in iter.as_list().unwrap() {
                                    self.set_ref(&var, i.clone(), &sub_scope)?;
                                    let temp = self.execute_scope(v.inner.clone(), &sub_scope)?;
                                    if let AstValue::Tuple((k, v)) = &temp {
                                        if let AstValue::String(k) = *k.clone() {
                                            attrs.insert(k.to_string(), *v.clone());
                                        }
                                    }
                                    if let AstValue::String(v) = &temp {
                                        content.push(
                                            AstElementContentType::Content(
                                                v.clone(),
                                            ),
                                        );
                                    }
                                    if let AstValue::Number(v) = &temp {
                                        content.push(
                                            AstElementContentType::Content(
                                                format!("{v}"),
                                            ),
                                        );
                                    }
                                    if let AstValue::Element(v) = temp {
                                        content.push(
                                            AstElementContentType::Children(v),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                AstElementContentType::Variable(v) => {
                    let result = self.get_ref(&v, current_scope)?;
                    if let AstValue::String(s) = &result.1.value {
                        content.push(AstElementContentType::Content(s.clone()));
                    }
                    if let AstValue::Number(s) = result.1.value {
                        content.push(AstElementContentType::Content(format!(
                            "{s}"
                        )));
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
