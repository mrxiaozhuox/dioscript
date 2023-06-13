use std::collections::HashMap;

use id_tree::{Node, NodeId, Tree, TreeBuilder};

use dioscript_parser::{
    ast::{
        CalculateMark, DioAstStatement, DioscriptAst, FunctionCall, FunctionDefine, LoopExecuteType,
    },
    element::{AstElement, AstElementContentType, Element, ElementContentType},
    error::{Error, RuntimeError},
    parser::CalcExpr,
    types::{AstValue, Value},
};
use stdlib::Exporter;

pub mod function;
pub mod stdlib;

pub struct Runtime {
    // variable content: use for save variable node-id.
    vars: HashMap<String, NodeId>,
    // scope tree: use for build scope structure.
    scope: Tree<ScopeType>,
    // root scope: root tree id.
    root_scope: NodeId,
    // functions content: use for save function node-id.
    functions: HashMap<String, NodeId>,
}

impl Runtime {
    pub fn new() -> Self {
        let mut scope = TreeBuilder::new().build();
        let root = scope
            .insert(Node::new(ScopeType::Block), id_tree::InsertBehavior::AsRoot)
            .expect("Scope init failed.");

        let mut this = Self {
            vars: HashMap::new(),
            scope,
            root_scope: root,
            functions: HashMap::new(),
        };

        this.add_function_list(crate::stdlib::value::export())
            .unwrap();
        this.add_function_list(crate::stdlib::console::export())
            .unwrap();
        this.add_function_list(crate::stdlib::root_export())
            .unwrap();

        this
    }

    pub fn add_function_list(&mut self, list: Exporter) -> Result<(), RuntimeError> {
        let namespace = list.0.clone();
        for (name, (function, param_num)) in list.1 {
            self.add_bind_function(namespace.clone(), &name, function, param_num)?;
        }
        Ok(())
    }

    pub fn add_bind_function(
        &mut self,
        namespace: Vec<String>,
        name: &str,
        func: function::Function,
        param_number: i16,
    ) -> Result<NodeId, RuntimeError> {
        let full_name = if namespace.is_empty() {
            name.to_string()
        } else {
            format!("{0}::{name}", namespace.join("::"))
        };
        let new_scope = self.scope.insert(
            Node::new(ScopeType::Function(FunctionType::RSF((func, param_number)))),
            id_tree::InsertBehavior::UnderNode(&self.root_scope),
        )?;
        self.functions
            .insert(full_name.to_string(), new_scope.clone());
        Ok(new_scope)
    }

    pub fn add_ds_function(
        &mut self,
        func: FunctionDefine,
    ) -> Result<(Option<NodeId>, Value), RuntimeError> {
        let full_name = func.name.clone();
        if let Some(name) = full_name {
            let new_scope = self.scope.insert(
                Node::new(ScopeType::Function(FunctionType::DSF(func.clone()))),
                id_tree::InsertBehavior::UnderNode(&self.root_scope),
            )?;
            self.functions.insert(name.to_string(), new_scope.clone());
            Ok((Some(new_scope), Value::Function(func)))
        } else {
            Ok((None, Value::Function(func)))
        }
    }

    pub fn execute(&mut self, code: &str) -> Result<Value, Error> {
        let ast = DioscriptAst::from_string(code)?;
        Ok(self.execute_ast(ast)?)
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
                    if let Value::Boolean(state) = state {
                        if state {
                            result = self.execute_scope(inner_ast, &sub_scope)?;
                            if !result.as_none() {
                                finish = true;
                            }
                        } else {
                            if let Some(otherwise) = otherwise {
                                result = self.execute_scope(otherwise, &sub_scope)?;
                                if !result.as_none() {
                                    finish = true;
                                }
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
                        LoopExecuteType::Iter { iter, var } => {
                            let iter = self.to_value(iter, current_scope)?;
                            if iter.value_name() == "list" {
                                for i in iter.as_list().unwrap() {
                                    self.set_ref(&var, i.clone(), &sub_scope)?;
                                    let res = self.execute_scope(data.inner.clone(), &sub_scope)?;
                                    if !res.as_none() {
                                        result = res;
                                        finish = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                DioAstStatement::FunctionCall(func) => {
                    let _result = self.execute_function(func, current_scope)?;
                }
                DioAstStatement::FunctionDefine(define) => {
                    let f = self.add_ds_function(define)?;
                    if f.0.is_none() {
                        return Err(RuntimeError::AnonymousFunctionInRoot);
                    }
                }
                _ => {}
            }
        }
        Ok(result)
    }

    fn to_value(&mut self, value: AstValue, current_scope: &NodeId) -> Result<Value, RuntimeError> {
        match value {
            AstValue::None => Ok(Value::None),
            AstValue::String(v) => Ok(Value::String(v)),
            AstValue::Number(v) => Ok(Value::Number(v)),
            AstValue::Boolean(v) => Ok(Value::Boolean(v)),
            AstValue::List(v) => {
                let mut res = Vec::new();
                for i in v {
                    let value = self.to_value(i, current_scope)?;
                    res.push(value);
                }
                Ok(Value::List(res))
            }
            AstValue::Dict(v) => {
                let mut res = HashMap::new();
                for (k, v) in v {
                    res.insert(k, self.to_value(v, current_scope)?);
                }
                Ok(Value::Dict(res))
            }
            AstValue::Tuple((a, b)) => {
                let a = self.to_value(*a, current_scope)?;
                let b = self.to_value(*b, current_scope)?;
                Ok(Value::Tuple((Box::new(a), Box::new(b))))
            }
            AstValue::Element(e) => {
                let element = self.execute_element(e, current_scope)?;
                Ok(Value::Element(element))
            }
            AstValue::Variable(n) => {
                let value = self.get_ref(&n, current_scope)?;
                Ok(value.1.value)
            }
            AstValue::VariableIndex((n, i)) => {
                let value = self.to_value(AstValue::Variable(n), current_scope)?;
                let index = self.to_value(*i, current_scope)?;
                let data = self.get_from_index(value, index)?;
                Ok(data)
            }
            AstValue::FunctionCaller(caller) => {
                let data = self.execute_function(caller, current_scope)?;
                Ok(data)
            }
            AstValue::FunctionDefine(define) => Ok(Value::Function(define)),
        }
    }

    fn execute_function(
        &mut self,
        caller: FunctionCall,
        current_scope: &NodeId,
    ) -> Result<Value, RuntimeError> {
        let name = if caller.namespace.is_empty() {
            caller.name.to_string()
        } else {
            format!("{}::{}", caller.namespace.join("::"), caller.name.clone())
        };
        let params = caller.arguments;
        let mut par = vec![];
        for i in params {
            let v = self.to_value(i, current_scope)?;
            par.push(v);
        }
        if let Some(ref_scope) = self.functions.get(&name) {
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
                    if let ScopeType::Function(v) = ref_node.data() {
                        match v {
                            FunctionType::DSF(f) => {
                                let f = f.clone();
                                let new_scope = self.scope.insert(
                                    Node::new(ScopeType::Block),
                                    id_tree::InsertBehavior::UnderNode(&self.root_scope),
                                )?;
                                match &f.params {
                                    dioscript_parser::ast::ParamsType::Variable(v) => {
                                        self.set_ref(v, Value::List(par), &new_scope)?;
                                    }
                                    dioscript_parser::ast::ParamsType::List(v) => {
                                        if v.len() != par.len() {
                                            return Err(RuntimeError::IllegalArgumentsNumber {
                                                need: v.len() as i16,
                                                provided: par.len() as i16,
                                            });
                                        }
                                        for (i, v) in v.iter().enumerate() {
                                            self.set_ref(
                                                v,
                                                par.get(i).unwrap().clone(),
                                                &new_scope,
                                            )?;
                                        }
                                    }
                                }
                                let result = self.execute_scope(f.inner, &new_scope)?;
                                return Ok(result);
                            }
                            FunctionType::RSF((f, need_param_num)) => {
                                if *need_param_num != -1 && (par.len() as i16) != *need_param_num {
                                    return Err(RuntimeError::IllegalArgumentsNumber {
                                        need: *need_param_num,
                                        provided: par.len() as i16,
                                    });
                                }
                                return Ok(f(self, par));
                            }
                        }
                    }
                }
            }
        }
        Err(RuntimeError::FunctionNotFound {
            name: name.to_string(),
        })
    }

    fn execute_calculate(
        &mut self,
        expr: CalcExpr,
        current_scope: &NodeId,
    ) -> Result<Value, RuntimeError> {
        match expr {
            CalcExpr::Value(v) => Ok(self.to_value(v, current_scope)?),
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
            CalcExpr::Mod(_, _) => Ok(Value::Boolean(false)),
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
        value: Value,
        current_scope: &NodeId,
    ) -> Result<NodeId, RuntimeError> {
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

    fn get_from_index(&self, value: Value, index: Value) -> Result<Value, RuntimeError> {
        match &value {
            Value::String(v) => {
                if let Value::Number(num) = index {
                    let num = num as usize;
                    let chars = v.chars();
                    let c = chars.collect::<Vec<char>>();
                    if c.len() >= num + 1 {
                        return Ok(Value::String(c[num].to_string()));
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
            Value::List(v) => {
                if let Value::Number(num) = index {
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
            Value::Dict(v) => {
                if let Value::String(key) = &index {
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
            Value::Tuple(v) => {
                if let Value::Number(num) = index {
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
    ) -> Result<Element, RuntimeError> {
        let mut attrs = HashMap::new();
        for i in element.attributes {
            let name = i.0;
            let data = i.1;
            attrs.insert(name, self.to_value(data, current_scope)?);
        }
        let mut content = vec![];
        for i in element.content {
            match i {
                AstElementContentType::Children(v) => {
                    let executed_element = self.execute_element(v, current_scope)?;
                    content.push(ElementContentType::Children(executed_element));
                }
                AstElementContentType::Content(v) => {
                    content.push(ElementContentType::Content(v));
                }
                AstElementContentType::Condition(v) => {
                    let value = self.execute_calculate(v.condition, current_scope)?;
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
                        if let Value::Tuple((k, v)) = &temp {
                            if let Value::String(k) = *k.clone() {
                                attrs.insert(k.to_string(), *v.clone());
                            }
                        }
                        if let Value::String(v) = &temp {
                            content.push(ElementContentType::Content(v.clone()));
                        }
                        if let Value::Number(v) = &temp {
                            content.push(ElementContentType::Content(format!("{v}")));
                        }
                        if let Value::Element(v) = temp {
                            content.push(ElementContentType::Children(v));
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
                                if let Value::Tuple((k, v)) = &temp {
                                    if let Value::String(k) = *k.clone() {
                                        attrs.insert(k.to_string(), *v.clone());
                                    }
                                }
                                if let Value::String(v) = &temp {
                                    content.push(ElementContentType::Content(v.clone()));
                                }
                                if let Value::Number(v) = &temp {
                                    content.push(ElementContentType::Content(format!("{v}")));
                                }
                                if let Value::Element(v) = temp {
                                    content.push(ElementContentType::Children(v));
                                }
                            }
                        },
                        LoopExecuteType::Iter { iter, var } => {
                            let iter = self.to_value(iter, current_scope)?;
                            if iter.value_name() == "list" {
                                for i in iter.as_list().unwrap() {
                                    self.set_ref(&var, i.clone(), &sub_scope)?;
                                    let temp = self.execute_scope(v.inner.clone(), &sub_scope)?;
                                    if let Value::Tuple((k, v)) = &temp {
                                        if let Value::String(k) = *k.clone() {
                                            attrs.insert(k.to_string(), *v.clone());
                                        }
                                    }
                                    if let Value::String(v) = &temp {
                                        content.push(ElementContentType::Content(v.clone()));
                                    }
                                    if let Value::Number(v) = &temp {
                                        content.push(ElementContentType::Content(format!("{v}")));
                                    }
                                    if let Value::Element(v) = temp {
                                        content.push(ElementContentType::Children(v));
                                    }
                                }
                            }
                        }
                    }
                }
                AstElementContentType::InlineExpr(v) => {
                    let result = self.execute_calculate(v, current_scope)?;
                    if let Value::String(s) = &result {
                        content.push(ElementContentType::Content(s.clone()));
                    }
                    if let Value::Number(s) = result {
                        content.push(ElementContentType::Content(format!("{s}")));
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

pub enum ScopeType {
    Block,
    Variable(Variable),
    Function(FunctionType),
}

pub enum FunctionType {
    DSF(FunctionDefine),
    RSF((function::Function, i16)),
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
    pub value: Value,
    pub counter: u32,
}
