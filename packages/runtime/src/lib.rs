use std::collections::HashMap;

use error::{Error, RuntimeError};
use id_tree::{Node, NodeId, Tree, TreeBuilder};

use dioscript_parser::{
    ast::{
        CalculateMark, DioAstStatement, DioscriptAst, FunctionCall, FunctionDefine,
        LoopExecuteType, ObjectDefine,
    },
    element::{AstElement, AstElementContentType},
    parser::{CalcExpr, LinkExpr},
    types::AstValue,
};
use types::{Element, ElementContentType, FunctionType, Value};

pub mod error;
pub mod function;
pub mod stdlib;
pub mod types;

pub struct Runtime {
    // variable content: use for save variable node-id.
    vars: HashMap<String, NodeId>,
    // scope tree: use for build scope structure.
    scope: Tree<ScopeType>,
    // root scope: root tree id.
    root_scope: NodeId,
    // function handle scope tree id.
    function_caller_scope: NodeId,
    // object prototype.
    objects: HashMap<String, ObjectDefine>,
}

impl Runtime {
    pub fn new() -> Self {
        let mut scope = TreeBuilder::new().build();
        let root = scope
            .insert(Node::new(ScopeType::Block), id_tree::InsertBehavior::AsRoot)
            .expect("Scope init failed.");

        let func_scope = scope
            .insert(
                Node::new(ScopeType::Block),
                id_tree::InsertBehavior::UnderNode(&root),
            )
            .expect("Scope init failed.");

        let mut this = Self {
            vars: HashMap::new(),
            scope,
            root_scope: root,
            function_caller_scope: func_scope,
            objects: HashMap::new(),
        };

        this.setup().expect("Runtime setup failed.");

        this
    }

    fn setup(&mut self) -> Result<(), RuntimeError> {
        let scope = self.root_scope.clone();

        let mut types: HashMap<String, HashMap<String, Value>> = HashMap::new();

        let functions = crate::stdlib::all();
        println!("{:?}", functions);
        for (target, list) in functions {
            match target {
                function::BindTarget::Root => {
                    for (name, val) in list {
                        self.set_var(&name, val, &scope)?;
                    }
                }
                _ => {
                    let target_name = target.to_string();
                    if types.contains_key(&target_name) {
                        let temp = types.get_mut(&target_name).unwrap();
                        let _ = list.into_iter().map(|(k, v)| temp.insert(k, v));
                    } else {
                        types.insert(target_name, list);
                    }
                }
            }
        }

        for (name, data) in types {
            self.set_var(&name, Value::Dict(data), &scope)?;
        }

        Ok(())
    }

    pub fn trace(&self) {
        println!("{:#?}", self.vars);
    }

    pub fn add_script_function(
        &mut self,
        func: FunctionDefine,
    ) -> Result<(Option<NodeId>, Value), RuntimeError> {
        let full_name = func.name.clone();
        if let Some(name) = full_name {
            let root_scope = self.root_scope.clone();
            let new_scope = self.set_var(
                &name,
                Value::Function(types::FunctionType::DefineFunction(func.clone())),
                &root_scope,
            )?;
            Ok((
                Some(new_scope),
                Value::Function(types::FunctionType::DefineFunction(func)),
            ))
        } else {
            Ok((
                None,
                Value::Function(types::FunctionType::DefineFunction(func)),
            ))
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

    fn execute_scope(
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
                    let _scope = self.set_var(&name, value, current_scope)?;
                }
                DioAstStatement::ReturnValue(r) => {
                    result = self.execute_calculate(r.clone(), current_scope)?;
                    result = self.deref_value(result, current_scope)?;
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
                                    self.set_var(&var, i.clone(), &sub_scope)?;
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
                    let f = self.add_script_function(define)?;
                    if f.0.is_none() {
                        return Err(RuntimeError::AnonymousFunctionInRoot);
                    }
                }
                DioAstStatement::ObjectDefine(object) => {
                    self.objects.insert(object.name.clone(), object);
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
                let element = self.to_element(e, current_scope)?;
                Ok(Value::Element(element))
            }
            AstValue::Variable(n) => {
                let value = self.get_var(&n, current_scope)?.1;
                Ok(value)
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
            AstValue::FunctionDefine(define) => {
                Ok(Value::Function(types::FunctionType::DefineFunction(define)))
            }
        }
    }

    fn deref_value(&self, value: Value, current_scope: &NodeId) -> Result<Value, RuntimeError> {
        match value {
            Value::List(list) => {
                let mut new = vec![];
                for i in list {
                    let v = self.deref_value(i, current_scope)?;
                    new.push(v);
                }
                Ok(Value::List(new))
            }
            Value::Dict(dict) => {
                let mut new = HashMap::new();
                for (k, v) in dict {
                    let v = self.deref_value(v, current_scope)?;
                    new.insert(k, v);
                }
                Ok(Value::Dict(new))
            }
            Value::Tuple(tuple) => {
                let first = self.deref_value(*tuple.0, current_scope)?;
                let second = self.deref_value(*tuple.1, current_scope)?;
                Ok(Value::Tuple((Box::new(first), Box::new(second))))
            }
            Value::Reference(id) => {
                let ref_node = self.scope.get(&id)?;

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
                    let data = ref_node.data();
                    if let ScopeType::Variable(v) = data {
                        return Ok(v.value.clone());
                    }
                }
                Err(RuntimeError::ReferenceNotFound { reference: id })
            }
            _ => Ok(value),
        }
    }

    fn execute_function(
        &mut self,
        caller: FunctionCall,
        current_scope: &NodeId,
    ) -> Result<Value, RuntimeError> {
        let runtime_scope = self.function_caller_scope.clone();
        let name = caller.name;
        let params = caller.arguments;
        let mut par = vec![];
        for i in params {
            let v = self.to_value(i, &current_scope)?;
            par.push(v);
        }

        let func = {
            let mut result = None;
            let info = self.get_var(&name, current_scope)?.1;
            if let Value::Function(f) = info {
                result = Some(f);
            }
            result
        };

        match func {
            Some(types::FunctionType::DefineFunction(f)) => {
                let f = f.clone();
                let new_scope = self.scope.insert(
                    Node::new(ScopeType::Block),
                    id_tree::InsertBehavior::UnderNode(&runtime_scope),
                )?;
                match &f.params {
                    dioscript_parser::ast::ParamsType::Variable(v) => {
                        self.set_var(&v, Value::List(par), &new_scope)?;
                    }
                    dioscript_parser::ast::ParamsType::List(v) => {
                        if v.len() != par.len() {
                            return Err(RuntimeError::IllegalArgumentsNumber {
                                need: v.len() as i16,
                                provided: par.len() as i16,
                            });
                        }
                        for (i, v) in v.iter().enumerate() {
                            self.set_var(v, par.get(i).unwrap().clone(), &new_scope)?;
                        }
                    }
                }
                let result = self.execute_scope(f.inner, &new_scope)?;
                return Ok(result);
            }
            Some(types::FunctionType::BindFunction((f, need_param_num))) => {
                if need_param_num != -1 && (par.len() as i32) != need_param_num {
                    return Err(RuntimeError::IllegalArgumentsNumber {
                        need: need_param_num as i16,
                        provided: par.len() as i16,
                    });
                }
                return Ok(f(self, par));
            }
            None => {
                return Err(RuntimeError::FunctionNotFound {
                    name: name.to_string(),
                });
            }
        }
    }

    fn execute_function_by_ft(
        &mut self,
        func: FunctionType,
        par: Vec<Value>,
        _current_scope: &NodeId,
    ) -> Result<Value, RuntimeError> {
        let runtime_scope = self.function_caller_scope.clone();
        match func {
            types::FunctionType::DefineFunction(f) => {
                let f = f.clone();
                let new_scope = self.scope.insert(
                    Node::new(ScopeType::Block),
                    id_tree::InsertBehavior::UnderNode(&runtime_scope),
                )?;
                match &f.params {
                    dioscript_parser::ast::ParamsType::Variable(v) => {
                        self.set_var(&v, Value::List(par), &new_scope)?;
                    }
                    dioscript_parser::ast::ParamsType::List(v) => {
                        if v.len() != par.len() {
                            return Err(RuntimeError::IllegalArgumentsNumber {
                                need: v.len() as i16,
                                provided: par.len() as i16,
                            });
                        }
                        for (i, v) in v.iter().enumerate() {
                            self.set_var(v, par.get(i).unwrap().clone(), &new_scope)?;
                        }
                    }
                }
                let result = self.execute_scope(f.inner, &new_scope)?;
                return Ok(result);
            }
            types::FunctionType::BindFunction((f, need_param_num)) => {
                if need_param_num != -1 && (par.len() as i32) != need_param_num {
                    return Err(RuntimeError::IllegalArgumentsNumber {
                        need: need_param_num as i16,
                        provided: par.len() as i16,
                    });
                }
                return Ok(f(self, par));
            }
        }
    }

    fn execute_calculate(
        &mut self,
        expr: CalcExpr,
        current_scope: &NodeId,
    ) -> Result<Value, RuntimeError> {
        match expr {
            CalcExpr::Value(v) => Ok(self.to_value(v, current_scope)?),
            CalcExpr::LinkExpr(v) => Ok(self.execute_link_expr(v, current_scope)?),
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

    fn execute_link_expr(
        &mut self,
        v: LinkExpr,
        current_scope: &NodeId,
    ) -> Result<Value, RuntimeError> {
        let mut this = self.to_value(v.this, current_scope)?;
        let list = v.list;
        for op in list {
            match op {
                dioscript_parser::parser::LinkExprPart::Field(field) => {
                    this = self.deref_value(this.clone(), current_scope)?;
                    match &this {
                        Value::List(list) => {
                            let index = field.parse::<usize>();
                            if let Ok(index) = index {
                                if list.len() - 1 >= index {
                                    this = list.get(index).unwrap().clone();
                                } else {
                                    return Err(RuntimeError::UnknownAttribute {
                                        attr: field,
                                        value: this.value_name(),
                                    });
                                }
                            } else {
                                return Err(RuntimeError::UnknownAttribute {
                                    attr: field,
                                    value: this.value_name(),
                                });
                            }
                        }
                        Value::Dict(dict) => {
                            if dict.contains_key(&field) {
                                this = dict.get(&field).unwrap().clone();
                            } else {
                                return Err(RuntimeError::UnknownAttribute {
                                    attr: field,
                                    value: this.value_name(),
                                });
                            }
                        }
                        Value::Tuple(tuple) => match field.as_str() {
                            "0" => {
                                this = *tuple.0.clone();
                            }
                            "1" => {
                                this = *tuple.1.clone();
                            }
                            _ => {
                                return Err(RuntimeError::UnknownAttribute {
                                    attr: field,
                                    value: this.value_name(),
                                });
                            }
                        },
                        Value::Element(element) => match field.as_str() {
                            "name" => {
                                this = Value::String(element.name.clone());
                            }
                            "attributes" => {
                                this = Value::Dict(element.attributes.clone());
                            }
                            "content" => {
                                let mut content = vec![];
                                for i in &element.content {
                                    match i {
                                        ElementContentType::Children(c) => {
                                            content.push(Value::Element(c.clone()));
                                        }
                                        ElementContentType::Content(c) => {
                                            content.push(Value::String(c.clone()));
                                        }
                                    }
                                }
                                this = Value::List(content);
                            }
                            _ => {
                                return Err(RuntimeError::UnknownAttribute {
                                    attr: field,
                                    value: this.value_name(),
                                });
                            }
                        },
                        _ => {
                            return Err(RuntimeError::UnknownAttribute {
                                attr: field,
                                value: this.value_name(),
                            });
                        }
                    }
                }
                dioscript_parser::parser::LinkExprPart::FunctionCall(call) => {
                    let root_scope = self.root_scope.clone();
                    match &this {
                        Value::String(content) => {
                            let mut pararms = vec![Value::String(content.to_string())];
                            for i in call.arguments {
                                let v = self.to_value(i, &current_scope)?;
                                pararms.push(v);
                            }
                            let v = self.get_var("string", &root_scope)?.1;
                            let v = self.deref_value(v, current_scope)?;
                            if let Value::Dict(v) = v {
                                if let Some(Value::Function(f)) = v.get(&call.name) {
                                    this = self.execute_function_by_ft(
                                        f.clone(),
                                        pararms,
                                        current_scope,
                                    )?;
                                }
                            }
                        }
                        Value::Number(_) => todo!(),
                        Value::Boolean(_) => todo!(),
                        Value::List(_) => todo!(),
                        Value::Dict(_) => todo!(),
                        Value::Tuple(_) => todo!(),
                        Value::Element(_) => todo!(),
                        Value::Function(_) => todo!(),
                        _ => {
                            unimplemented!()
                        }
                    }
                }
            }
        }
        Ok(self.deref_value(this, current_scope)?)
    }

    fn get_var(&self, name: &str, current_scope: &NodeId) -> Result<(NodeId, Value), RuntimeError> {
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
                    let data = ref_node.data();
                    if let ScopeType::Variable(v) = data {
                        return Ok((ref_node_id.clone(), v.value.clone()));
                    }
                }
            }
        }
        Err(RuntimeError::VariableNotFound {
            name: name.to_string(),
        })
    }

    fn get_mut_var(
        &mut self,
        name: &str,
        current_scope: &NodeId,
    ) -> Result<&mut Value, RuntimeError> {
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
                    let ref_node = self.scope.get_mut(ref_scope)?;
                    if let ScopeType::Variable(v) = ref_node.data_mut() {
                        return Ok(&mut v.value);
                    }
                }
            }
        }
        Err(RuntimeError::VariableNotFound {
            name: name.to_string(),
        })
    }

    fn get_mut_var_by_id(
        &mut self,
        id: &NodeId,
        current_scope: &NodeId,
    ) -> Result<&mut Value, RuntimeError> {
        let ref_node = self.scope.get(&id)?;

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
            let ref_node = self.scope.get_mut(&id)?;
            if let ScopeType::Variable(v) = ref_node.data_mut() {
                return Ok(&mut v.value);
            }
        }
        Err(RuntimeError::ReferenceNotFound {
            reference: id.clone(),
        })
    }

    fn set_var(
        &mut self,
        name: &str,
        value: Value,
        current_scope: &NodeId,
    ) -> Result<NodeId, RuntimeError> {
        let new_node_scope = if let Some(scope) = self.vars.get(name) {
            let scope = self.scope.get(scope)?.parent().unwrap().clone();
            scope
        } else {
            current_scope.clone()
        };

        let value = match value {
            Value::List(list) => {
                let mut result = vec![];
                for (i, v) in list.iter().enumerate() {
                    if let Value::Reference(_) = v {
                        // ignore
                        result.push(v.clone());
                    } else {
                        let name = format!("{name}[{i}]");
                        let id = self.set_var(&name, v.clone(), &new_node_scope)?;
                        result.push(Value::Reference(id));
                    }
                }
                Value::List(result)
            }
            Value::Dict(dict) => {
                let mut result = HashMap::new();
                for (k, v) in dict {
                    if let Value::Reference(_) = v {
                        // ignore
                        result.insert(k, v);
                    } else {
                        let name = format!("{name}[{k}]");
                        let id = self.set_var(&name, v.clone(), &new_node_scope)?;
                        result.insert(k, Value::Reference(id));
                    }
                }
                Value::Dict(result)
            }
            Value::Tuple(tuple) => {
                let first = {
                    if let Value::Reference(_) = *tuple.0.clone() {
                        // ignore
                        Box::new(*tuple.0)
                    } else {
                        let name = format!("{name}[0]");
                        let id = self.set_var(&name, *tuple.0, &new_node_scope)?;
                        Box::new(Value::Reference(id))
                    }
                };
                let second = {
                    if let Value::Reference(_) = *tuple.1.clone() {
                        // ignore
                        Box::new(*tuple.1)
                    } else {
                        let name = format!(".{name}.1");
                        let id = self.set_var(&name, *tuple.1, current_scope)?;
                        Box::new(Value::Reference(id))
                    }
                };
                Value::Tuple((first, second))
            }
            _ => value,
        };
        let scope = if let Some(scope) = self.vars.get(name) {
            let vars = self.scope.get_mut(scope)?.data_mut();
            if let ScopeType::Variable(v) = vars {
                v.value = value;
                v.counter += 1;
            }
            scope.clone()
        } else {
            let new_scope = self.scope.insert(
                Node::new(ScopeType::Variable(Variable { value, counter: 0 })),
                id_tree::InsertBehavior::UnderNode(current_scope),
            )?;
            self.vars.insert(name.to_string(), new_scope.clone());
            new_scope
        };
        return Ok(scope);
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

    fn to_element(
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
                    let executed_element = self.to_element(v, current_scope)?;
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
                                    self.set_var(&var, i.clone(), &sub_scope)?;
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
