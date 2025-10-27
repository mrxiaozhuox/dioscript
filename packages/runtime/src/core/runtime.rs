use std::collections::{HashMap, HashSet};

use dioscript_parser::{
    ast::{
        CalculateMark, DioAstStatement, FunctionCall, FunctionDefine, FunctionName, LoopExecuteType,
    },
    element::{AstElement, AstElementContentType},
    parser::{CalcExpr, LinkExpr, LinkExprPart},
    types::AstValue,
};
use uuid::Uuid;

use super::{
    error::RuntimeError,
    io::output::{ConsoleOutputHandler, OutputHandler},
    module::{self, ModuleGenerator, ModuleItem, RustyExecutor},
    names::RESERVED_KEYWORDS,
    scope::Scope,
    types::{self, Element, ElementContentType, FunctionType, Value},
    DataType,
};

pub struct Runtime {
    // variable content: use for save variable node-id.
    pub scopes: Vec<Scope>,
    // scope tree: use for build scope structure.
    pub data: HashMap<Uuid, DataType>,
    // module included.
    pub modules: HashMap<String, module::ModuleItem>,
    // namespace using list
    pub namespace_use: HashMap<String, Vec<String>>,
    // output_handler
    pub output_handler: Box<dyn OutputHandler>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            scopes: Vec::new(),
            data: HashMap::new(),
            modules: HashMap::new(),
            namespace_use: HashMap::new(),
            output_handler: Box::new(ConsoleOutputHandler),
        }
    }
}

impl Runtime {
    pub fn bind_module(&mut self, name: &str, module: ModuleGenerator) {
        self.modules
            .insert(name.to_string(), module.to_module_item());
    }

    pub fn add_script_function(
        &mut self,
        func: FunctionDefine,
    ) -> Result<(Option<Uuid>, Value), RuntimeError> {
        let full_name = func.name.clone();
        if let Some(name) = full_name {
            let new_scope = self.create_var(
                &name,
                Value::Function(types::FunctionType::DScript((func.clone(), HashMap::new()))),
            )?;

            Ok((
                Some(new_scope),
                Value::Function(types::FunctionType::DScript((func, HashMap::new()))),
            ))
        } else {
            Err(RuntimeError::AnonymousFunctionInRoot)
        }
    }

    pub fn collect_free_vars(&self, func: &FunctionDefine) -> HashMap<String, Uuid> {
        let func_name = &func.name;

        let mut env = HashMap::new();

        for scope in self.scopes.iter().rev() {
            for (k, id) in &scope.data {
                if let Some(name) = func_name {
                    if k == name {
                        continue;
                    }
                }
                env.entry(k.clone()).or_insert(*id);
            }
            if scope.isolate {
                break;
            }
        }

        env
    }

    // WARNING:
    // this should be a better solution, but will only use for new version, so just comment this part
    //
    // pub fn collect_free_vars(&self, func: &FunctionDefine) -> HashMap<String, Uuid> {
    //     let mut bound: HashSet<String> = func.params.iter().cloned().collect();
    //     if let Some(v) = &func.variadic_param {
    //         bound.insert(v.clone());
    //     }
    //     let mut free: HashSet<String> = HashSet::new();
    //
    //     fn walk_stmt(
    //         st: &DioAstStatement,
    //         bound: &mut HashSet<String>,
    //         free: &mut HashSet<String>,
    //     ) {
    //         match st {
    //             DioAstStatement::VariableAss(var) => {
    //                 walk_expr(&var.expr, bound, free);
    //                 bound.insert(var.name.clone());
    //             }
    //
    //             DioAstStatement::FunctionDefine(fd) => {
    //                 if let Some(name) = &fd.name {
    //                     bound.insert(name.clone());
    //                 }
    //                 for inner in &fd.inner {
    //                     walk_stmt(inner, bound, free);
    //                 }
    //             }
    //
    //             DioAstStatement::IfStatement(ifst) => {
    //                 walk_expr(&ifst.condition, bound, free);
    //                 for s in &ifst.inner {
    //                     walk_stmt(s, bound, free);
    //                 }
    //                 if let Some(oth) = &ifst.otherwise {
    //                     for s in oth {
    //                         walk_stmt(s, bound, free);
    //                     }
    //                 }
    //             }
    //             DioAstStatement::LoopStatement(lp) => {
    //                 match &lp.execute_type {
    //                     LoopExecuteType::Conditional(e) => {
    //                         walk_expr(e, bound, free);
    //                     }
    //                     LoopExecuteType::Iter { iter, var } => {
    //                         walk_expr(iter, bound, free);
    //                         bound.insert(var.clone());
    //                     }
    //                 }
    //                 for s in &lp.inner {
    //                     walk_stmt(s, bound, free);
    //                 }
    //             }
    //
    //             DioAstStatement::FunctionCall(fc) => {
    //                 for e in &fc.arguments {
    //                     walk_expr(e, bound, free);
    //                 }
    //             }
    //             DioAstStatement::ReturnValue(e) | DioAstStatement::CalcExpr(e) => {
    //                 walk_expr(e, bound, free)
    //             }
    //             _ => {}
    //         }
    //     }
    //
    //     fn walk_expr(expr: &CalcExpr, bound: &mut HashSet<String>, free: &mut HashSet<String>) {
    //         match expr {
    //             CalcExpr::Value(AstValue::Variable(name)) => {
    //                 if !bound.contains(name) {
    //                     free.insert(name.clone());
    //                 }
    //             }
    //             CalcExpr::Value(AstValue::VariableIndex((name, idx))) => {
    //                 if !bound.contains(name) {
    //                     free.insert(name.clone());
    //                 }
    //                 walk_expr(idx, bound, free);
    //             }
    //             CalcExpr::Value(_) => {}
    //             CalcExpr::LinkExpr(link) => {
    //                 // link.this
    //                 if let AstValue::Variable(n) = &link.this {
    //                     if !bound.contains(n) {
    //                         free.insert(n.clone());
    //                     }
    //                 }
    //                 for part in &link.list {
    //                     if let LinkExprPart::FunctionCall(fc) = part {
    //                         for arg in &fc.arguments {
    //                             walk_expr(arg, bound, free);
    //                         }
    //                     }
    //                 }
    //             }
    //             CalcExpr::Add(l, r)
    //             | CalcExpr::Sub(l, r)
    //             | CalcExpr::Mul(l, r)
    //             | CalcExpr::Div(l, r)
    //             | CalcExpr::Mod(l, r)
    //             | CalcExpr::Eq(l, r)
    //             | CalcExpr::Ne(l, r)
    //             | CalcExpr::Gt(l, r)
    //             | CalcExpr::Lt(l, r)
    //             | CalcExpr::Ge(l, r)
    //             | CalcExpr::Le(l, r)
    //             | CalcExpr::And(l, r)
    //             | CalcExpr::Or(l, r) => {
    //                 walk_expr(l, bound, free);
    //                 walk_expr(r, bound, free);
    //             }
    //         }
    //     }
    //
    //     for s in &func.inner {
    //         walk_stmt(s, &mut bound, &mut free);
    //     }
    //
    //     free.into_iter()
    //         .filter_map(|name| self.get_var(&name).ok().map(|(id, _v)| (name, id)))
    //         .collect()
    // }

    pub fn get_ref_value(&self, id: &Uuid) -> Result<Value, RuntimeError> {
        if let Some(DataType::Variable(var)) = self.data.get(id) {
            return Ok(var.clone());
        }
        Err(RuntimeError::UnknownPointer {
            pointer: id.to_string(),
        })
    }

    pub fn set_ref_value(&mut self, id: &Uuid, value: Value) -> Result<(), RuntimeError> {
        if let Some(DataType::Variable(var)) = self.data.get_mut(id) {
            *var = value;
            return Ok(());
        }
        Err(RuntimeError::UnknownPointer {
            pointer: id.to_string(),
        })
    }

    // collect function define from statments
    // only collect named function
    pub fn collect_functions(
        &mut self,
        statements: &[DioAstStatement],
    ) -> Result<(), RuntimeError> {
        for statement in statements {
            if let DioAstStatement::FunctionDefine(fd) = statement {
                if fd.name.is_some() {
                    self.add_script_function(fd.clone())?;
                }
            }
        }

        Ok(())
    }

    pub fn enter_scope(&mut self, i: bool) {
        let scope = if i { Scope::fun() } else { Scope::gen() };
        self.scopes.push(scope);
    }

    pub fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn execute_scope(
        &mut self,
        statements: Vec<DioAstStatement>,
    ) -> Result<Value, RuntimeError> {
        // enter a new scope
        self.enter_scope(false);

        let result = self.execute_scope_without_new_scope(statements)?;

        self.leave_scope();

        Ok(result)
    }

    pub fn execute_isolate_scope(
        &mut self,
        statements: Vec<DioAstStatement>,
    ) -> Result<Value, RuntimeError> {
        // enter a new scope
        self.enter_scope(true);

        let result = self.execute_scope_without_new_scope(statements)?;

        self.leave_scope();

        Ok(result)
    }

    pub fn execute_scope_without_new_scope(
        &mut self,
        statements: Vec<DioAstStatement>,
    ) -> Result<Value, RuntimeError> {
        // result: return value
        // finish: interrupt status
        let mut result: Value = Value::None;
        let mut finish = false;

        // collect current level functions.
        self.collect_functions(&statements)?;

        for v in statements {
            if finish {
                break;
            }
            match v {
                DioAstStatement::ModuleUse(u) => {
                    let u = u.0;
                    let last = u.last().unwrap();
                    self.namespace_use.insert(last.to_string(), u.clone());
                }
                DioAstStatement::VariableAss(var) => {
                    let name = var.name.clone();
                    let value = var.expr.clone();
                    let value = self.execute_calculate(value)?;
                    if var.new {
                        let _scope = self.create_var(&name, value)?;
                    } else {
                        let _scope = self.set_var(&name, value)?;
                    }
                }
                DioAstStatement::ReturnValue(r) => {
                    result = self.execute_calculate(r.clone())?;
                    result = self.deref_value(result)?;
                    finish = true;
                }
                DioAstStatement::IfStatement(cond) => {
                    let condition_expr = cond.condition.clone();
                    let inner_ast = cond.inner.clone();
                    let otherwise = cond.otherwise.clone();
                    let state = self.execute_calculate(condition_expr)?;
                    if let Value::Boolean(state) = state {
                        if state {
                            result = self.execute_scope(inner_ast)?;
                            if !result.as_none() {
                                finish = true;
                            }
                        } else if let Some(otherwise) = otherwise {
                            result = self.execute_scope(otherwise)?;
                            if !result.as_none() {
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
                    let execute_type = data.execute_type;
                    match execute_type {
                        LoopExecuteType::Conditional(cond) => loop {
                            let cond = cond.clone();
                            let state = self.execute_calculate(cond)?;
                            let state = state.to_boolean_data();
                            if !state {
                                break;
                            } else {
                                let res = self.execute_scope(data.inner.clone())?;
                                if !res.as_none() {
                                    result = res;
                                    finish = true;
                                    break;
                                }
                            }
                        },
                        LoopExecuteType::Iter { iter, var } => {
                            let iter = self.execute_calculate(iter)?;
                            if iter.value_name() == "list" {
                                for i in iter.as_list().unwrap() {
                                    self.create_var(&var, i.clone())?;
                                    let res = self.execute_scope(data.inner.clone())?;
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
                    let _result = self.execute_function(func)?;
                }
                DioAstStatement::FunctionDefine(define) => {
                    let f = self.add_script_function(define)?;
                    if f.0.is_none() {
                        return Err(RuntimeError::AnonymousFunctionInRoot);
                    }
                }
                DioAstStatement::CalcExpr(expr) => {
                    let _value = self.execute_calculate(expr)?;
                }
                _ => {}
            }
        }
        Ok(result)
    }

    pub fn to_value(&mut self, value: AstValue) -> Result<Value, RuntimeError> {
        match value {
            AstValue::None => Ok(Value::None),
            AstValue::String(v) => Ok(Value::String(v)),
            AstValue::Number(v) => Ok(Value::Number(v)),
            AstValue::Boolean(v) => Ok(Value::Boolean(v)),
            AstValue::List(v) => {
                let mut res = Vec::new();
                for i in v {
                    let value = self.execute_calculate(i)?;
                    res.push(value);
                }
                Ok(Value::List(res))
            }
            AstValue::Dict(v) => {
                let mut res = HashMap::new();
                for (k, v) in v {
                    res.insert(k, self.execute_calculate(v)?);
                }
                Ok(Value::Dict(res))
            }
            AstValue::Tuple((a, b)) => {
                let a = self.execute_calculate(*a)?;
                let b = self.execute_calculate(*b)?;
                Ok(Value::Tuple((Box::new(a), Box::new(b))))
            }
            AstValue::Element(e) => {
                let element = self.to_element(e)?;
                Ok(Value::Element(element))
            }
            AstValue::Variable(n) => {
                let value = self.get_var(&n)?.1;
                let value = self.deref_value(value)?;
                Ok(value)
            }
            AstValue::VariableIndex((n, i)) => {
                let value = self.to_value(AstValue::Variable(n))?;
                let index = self.execute_calculate(*i)?;
                let data = self.get_from_index(value, index)?;
                Ok(data)
            }
            AstValue::FunctionCaller(caller) => {
                let data = self.execute_function(caller)?;
                Ok(data)
            }
            AstValue::FunctionDefine(define) => {
                let env = if define.name.is_none() {
                    self.collect_free_vars(&define)
                } else {
                    HashMap::new()
                };
                Ok(Value::Function(types::FunctionType::DScript((define, env))))
            }
            AstValue::Reference(target) => {
                let (id, _) = self.get_var(&target)?;
                Ok(Value::Reference(id))
            }
        }
    }

    pub fn deref_value(&self, value: Value) -> Result<Value, RuntimeError> {
        self.deref_inner(value, HashSet::new())
    }

    // use for check CircularReference
    fn deref_inner(&self, value: Value, mut seen: HashSet<Uuid>) -> Result<Value, RuntimeError> {
        match value {
            Value::List(list) => {
                let mut new = vec![];
                for i in list {
                    let v = self.deref_value(i)?;
                    new.push(v);
                }
                Ok(Value::List(new))
            }
            Value::Dict(dict) => {
                let mut new = HashMap::new();
                for (k, v) in dict {
                    let v = self.deref_value(v)?;
                    new.insert(k, v);
                }
                Ok(Value::Dict(new))
            }
            Value::Tuple(tuple) => {
                let first = self.deref_value(*tuple.0)?;
                let second = self.deref_value(*tuple.1)?;
                Ok(Value::Tuple((Box::new(first), Box::new(second))))
            }
            Value::Reference(id) => {
                if !seen.insert(id) {
                    return Err(RuntimeError::CircularReference);
                }
                let v = self.get_ref_value(&id)?;
                self.deref_inner(v, seen)
            }
            // TODO: Element
            _ => Ok(value),
        }
    }

    pub fn execute_function(&mut self, caller: FunctionCall) -> Result<Value, RuntimeError> {
        let name = caller.name;
        let params = caller.arguments;
        let mut par = vec![];
        for i in params {
            let v = self.execute_calculate(i)?;
            par.push(v);
        }

        let func = self.get_function(name)?;

        self.execute_function_by_ft(func, par)
    }

    pub fn execute_function_by_ft(
        &mut self,
        func: FunctionType,
        par: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        match func {
            types::FunctionType::DScript((f, env)) => {
                let f = f.clone();

                // Enter a new function scope
                self.enter_scope(true);

                for (k, id) in &env {
                    self.scopes.last_mut().unwrap().data.insert(k.clone(), *id);
                }

                // Set the function parameters in the new scope
                let fixed_len = f.params.len();
                let provided_len = par.len();
                let has_variadic = f.variadic_param.is_some();

                if (!has_variadic && provided_len != fixed_len)
                    || (has_variadic && provided_len < fixed_len)
                {
                    self.leave_scope();
                    return Err(RuntimeError::IllegalArgumentsNumber {
                        need: fixed_len as i16,
                        provided: provided_len as i16,
                    });
                }

                for (idx, name) in f.params.iter().enumerate() {
                    self.create_var(name, par[idx].clone())?;
                }

                if let Some(var_name) = &f.variadic_param {
                    let rest_slice = par[fixed_len..].to_vec();
                    self.create_var(var_name, Value::List(rest_slice))?;
                }

                // Execute the function body
                let result = self.execute_scope(f.inner)?;

                // Leave the function scope
                self.leave_scope();

                Ok(result)
            }
            types::FunctionType::Rusty((f, need_param_num)) => {
                if need_param_num != -1 && (par.len() as i32) != need_param_num {
                    return Err(RuntimeError::IllegalArgumentsNumber {
                        need: need_param_num as i16,
                        provided: par.len() as i16,
                    });
                }
                f(RustyExecutor::bind(self), par)
            }
        }
    }

    pub fn get_function(&self, name: FunctionName) -> Result<FunctionType, RuntimeError> {
        match name {
            FunctionName::Single(name) => {
                for scope in self.scopes.iter().rev() {
                    if let Some(id) = scope.data.get(&name) {
                        if let Some(DataType::Variable(Value::Function(f))) = self.data.get(id) {
                            return Ok(f.clone());
                        }
                    }
                }

                let info = self.get_var(&name);
                if let Ok((_, Value::Function(f))) = info {
                    Ok(f)
                } else {
                    let function = self.get_module_value(vec![name.clone()]);
                    if let Ok(ModuleItem::Function(f)) = function {
                        Ok(f)
                    } else {
                        Err(RuntimeError::FunctionNotFound { name })
                    }
                }
            }
            FunctionName::Namespace(namespace) => {
                let v = self.get_module_value(namespace.clone())?;
                if let ModuleItem::Function(f) = v {
                    Ok(f)
                } else {
                    Err(RuntimeError::FunctionNotFound {
                        name: namespace.join("::"),
                    })
                }
            }
        }
    }

    pub fn get_module_value(&self, mut namespace: Vec<String>) -> Result<ModuleItem, RuntimeError> {
        let data = self.load_from_module(namespace.clone());
        match data {
            Ok(v) => Ok(v),
            Err(_) => {
                let v = self.namespace_use.get(&namespace[0]);
                if let Some(used) = v {
                    if used.last().unwrap() == &namespace[0] {
                        namespace.remove(0);
                    }
                    let module_path = used.iter().chain(namespace.iter()).cloned().collect();
                    let v = self.load_from_module(module_path)?;
                    Ok(v)
                } else {
                    Err(RuntimeError::ModuleNotFound {
                        module: namespace[0].to_string(),
                    })
                }
            }
        }
    }

    pub fn load_from_module(&self, namespace: Vec<String>) -> Result<ModuleItem, RuntimeError> {
        let map = &self.modules;
        let mut cur_item: ModuleItem = map
            .get(&namespace[0])
            .ok_or(RuntimeError::ModuleNotFound {
                module: namespace[0].to_string(),
            })?
            .clone();

        for ns in &namespace[1..] {
            match cur_item {
                ModuleItem::SubModule(sub_info) => {
                    let sub_map = sub_info.0;
                    cur_item = sub_map
                        .get(ns)
                        .ok_or(RuntimeError::ModulePartNotFound {
                            part: ns.to_string(),
                            module: namespace[0].to_string(),
                        })?
                        .clone();
                }
                _ => {
                    return Err(RuntimeError::ModulePartNotFound {
                        part: ns.to_string(),
                        module: namespace[0].to_string(),
                    })
                }
            }
        }
        let r = cur_item.clone();
        Ok(r)
    }

    pub fn execute_calculate(&mut self, expr: CalcExpr) -> Result<Value, RuntimeError> {
        match expr {
            CalcExpr::Value(v) => Ok(self.to_value(v)?),
            CalcExpr::LinkExpr(v) => Ok(self.execute_link_expr(v)?),
            CalcExpr::Add(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::Plus)
            }
            CalcExpr::Sub(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::Minus)
            }
            CalcExpr::Mul(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::Multiply)
            }
            CalcExpr::Div(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::Divide)
            }
            CalcExpr::Mod(_l, _r) => {
                // let l = self.execute_calculate(*l)?;
                // let r = self.execute_calculate(*r)?;
                // l.calc(&r, CalculateMark::Mod)
                Ok(Value::Boolean(false))
            }
            CalcExpr::Eq(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::Equal)
            }
            CalcExpr::Ne(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::NotEqual)
            }
            CalcExpr::Gt(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::Large)
            }
            CalcExpr::Lt(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::Small)
            }
            CalcExpr::Ge(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::LargeOrEqual)
            }
            CalcExpr::Le(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::SmallOrEqual)
            }
            CalcExpr::And(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::And)
            }
            CalcExpr::Or(l, r) => {
                let l = self.execute_calculate(*l)?;
                let r = self.execute_calculate(*r)?;
                l.calc(&r, CalculateMark::Or)
            }
        }
    }

    pub fn execute_link_expr(&mut self, v: LinkExpr) -> Result<Value, RuntimeError> {
        // if `this` was a var, get the ref-id
        let meta_this = self.to_value(v.this.clone())?;
        let mut this = if let AstValue::Variable(var_name) = &v.this {
            let (id, _) = self.get_var(var_name)?;
            Value::Reference(id)
        } else {
            meta_this.clone()
        };

        let list = v.list;
        for op in list {
            match op {
                dioscript_parser::parser::LinkExprPart::Field(field) => {
                    this = self.deref_value(this.clone())?;
                    match &this {
                        // Element:
                        //  name: element name [string]
                        //  attributes: attribute list k:v [dict]
                        //  content: content list [list]
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
                    let function_name = call.name;
                    let mut params = vec![this.clone()];
                    for i in call.arguments {
                        let v = self.execute_calculate(i)?;
                        params.push(v);
                    }

                    #[allow(warnings)]
                    if let Some(m) = self.modules.get(&meta_this.value_name()) {
                        if let ModuleItem::SubModule(sub) = m {
                            if let Some(v) = sub.0.get(&function_name.as_single()) {
                                if let ModuleItem::Function(f) = v {
                                    this = self.execute_function_by_ft(f.clone(), params)?;
                                    return self.deref_value(this);
                                }
                            }
                        }
                    }
                    return Err(RuntimeError::FunctionNotFound {
                        name: function_name.to_string(),
                    });
                }
            }
        }
        self.deref_value(this)
    }

    // get variable value:
    pub fn get_var(&self, name: &str) -> Result<(Uuid, Value), RuntimeError> {
        for scope in self.scopes.iter().rev() {
            if let Some(uuid) = scope.data.get(name) {
                if let Some(data_type) = self.data.get(uuid) {
                    let value = data_type.as_variable().unwrap();
                    return Ok((*uuid, value));
                }
            }
            if scope.isolate {
                break;
            }
        }
        Err(RuntimeError::VariableNotFound {
            name: name.to_string(),
        })
    }

    #[allow(irrefutable_let_patterns)]
    pub fn create_var(&mut self, name: &str, value: Value) -> Result<Uuid, RuntimeError> {
        if RESERVED_KEYWORDS.contains(&name) {
            return Err(RuntimeError::UsingReservedKeyword {
                keyword: name.to_string(),
            });
        }

        let current_scope = self.scopes.last_mut().ok_or(RuntimeError::ScopeNotFound)?;

        if current_scope.data.contains_key(name) {
            Err(RuntimeError::VariableAlreadyDefined { name: name.into() })
        } else {
            let id = Uuid::new_v4();
            self.data.insert(id, DataType::Variable(value));
            current_scope.data.insert(name.to_string(), id);
            Ok(id)
        }
    }

    fn follow_ref<'a>(&'a self, mut id: &'a Uuid) -> Result<&'a Uuid, RuntimeError> {
        let mut hops = 0;
        loop {
            match self.data.get(id) {
                Some(DataType::Variable(Value::Reference(next))) => {
                    id = next;
                    hops += 1;
                    if hops > 35 {
                        return Err(RuntimeError::CircularReference);
                    }
                }
                Some(_) => return Ok(id),
                None => {
                    return Err(RuntimeError::UnknownPointer {
                        pointer: id.to_string(),
                    })
                }
            }
        }
    }

    #[allow(irrefutable_let_patterns)]
    pub fn set_var(&mut self, name: &str, value: Value) -> Result<Uuid, RuntimeError> {
        let (cur_id, _) = self.get_var(name)?;

        match value {
            Value::Reference(new_id) => {
                self.scopes
                    .iter_mut()
                    .rev()
                    .take_while(|s| !s.isolate)
                    .find_map(|scope| scope.data.get_mut(name))
                    .map(|slot| *slot = new_id)
                    .ok_or_else(|| RuntimeError::VariableNotFound { name: name.into() })?;

                Ok(new_id)
            }
            v => {
                let target_id = *self.follow_ref(&cur_id)?;
                self.set_ref_value(&target_id, v)?;
                Ok(target_id)
            }
        }
    }

    #[allow(dead_code)]
    pub fn create_data(&mut self, data: Value) -> Result<Uuid, RuntimeError> {
        let id = Uuid::new_v4();
        self.data.insert(id, DataType::Variable(data));
        Ok(id)
    }

    pub fn get_from_index(&self, value: Value, index: Value) -> Result<Value, RuntimeError> {
        let value = self.deref_value(value)?;
        match &value {
            Value::String(v) => {
                if let Value::Number(num) = index {
                    let num = num as usize;
                    let chars = v.chars();
                    let c = chars.collect::<Vec<char>>();
                    if c.len() > num {
                        Ok(Value::String(c[num].to_string()))
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
                    if v.len() > num {
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
                            index: key.to_string(),
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

    pub fn to_element(&mut self, element: AstElement) -> Result<Element, RuntimeError> {
        let mut attrs = HashMap::new();
        for i in element.attributes {
            let name = i.0;
            let data = i.1;
            attrs.insert(name, self.to_value(data)?);
        }
        let mut content = vec![];
        for i in element.content {
            match i {
                AstElementContentType::Children(v) => {
                    let executed_element = self.to_element(v)?;
                    content.push(ElementContentType::Children(executed_element));
                }
                AstElementContentType::Content(v) => {
                    content.push(ElementContentType::Content(v));
                }
                AstElementContentType::Condition(v) => {
                    let value = self.execute_calculate(v.condition)?;
                    if let Value::Boolean(b) = value {
                        let mut temp = Value::None;
                        if b {
                            temp = self.execute_scope(v.inner)?;
                        } else if let Some(otherwise) = v.otherwise {
                            temp = self.execute_scope(otherwise)?;
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
                    let execute_type = v.execute_type;
                    match execute_type {
                        LoopExecuteType::Conditional(cond) => loop {
                            let cond = cond.clone();
                            let state = self.execute_calculate(cond)?;
                            let state = state.to_boolean_data();
                            if !state {
                                break;
                            } else {
                                let temp = self.execute_scope(v.inner.clone())?;
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
                            let iter = self.execute_calculate(iter)?;
                            if iter.value_name() == "list" {
                                for i in iter.as_list().unwrap() {
                                    self.create_var(&var, i.clone())?;
                                    let temp = self.execute_scope(v.inner.clone())?;
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
                    let result = self.execute_calculate(v)?;
                    let new_content = Self::ast_element_value_to_content(result);
                    content.extend(new_content);
                }
            }
        }
        Ok(Element {
            name: element.name,
            attributes: attrs,
            content,
        })
    }

    // use for handle different value type -> element content.
    pub fn ast_element_value_to_content(result: Value) -> Vec<ElementContentType> {
        let mut content: Vec<ElementContentType> = vec![];
        match result {
            Value::None => content.push(ElementContentType::Content("none".to_string())),
            Value::String(s) => content.push(ElementContentType::Content(s.clone())),
            Value::Number(s) => content.push(ElementContentType::Content(format!("{s}"))),
            Value::Boolean(s) => content.push(ElementContentType::Content(s.to_string())),
            Value::Element(s) => content.push(ElementContentType::Children(s.clone())),
            Value::List(s) => {
                for i in s {
                    content.extend(Self::ast_element_value_to_content(i));
                }
            }
            _ => content.push(ElementContentType::Content(result.to_string())),
        };
        content
    }
}
