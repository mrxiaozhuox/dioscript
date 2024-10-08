use std::collections::HashMap;

use error::{Error, RuntimeError};

use dioscript_parser::{
    ast::{
        CalculateMark, DioAstStatement, DioscriptAst, FunctionCall, FunctionDefine, FunctionName,
        LoopExecuteType,
    },
    element::{AstElement, AstElementContentType},
    parser::{CalcExpr, LinkExpr},
    types::AstValue,
};
use module::{ModuleGenerator, ModuleItem};
use types::{Element, ElementContentType, FunctionType, Value};
use uuid::Uuid;

pub mod error;
pub mod module;
pub mod stdlib;
pub mod types;

pub struct Runtime {
    // variable content: use for save variable node-id.
    scopes: Vec<Scope>,
    // scope tree: use for build scope structure.
    data: HashMap<Uuid, DataType>,
    // module included.
    modules: HashMap<String, module::ModuleItem>,
    // namespace using list
    namespace_use: HashMap<String, Vec<String>>,
}

impl Runtime {
    pub fn new() -> Self {
        let mut this = Self {
            scopes: vec![],
            data: HashMap::new(),
            modules: Default::default(),
            namespace_use: Default::default(),
        };

        this.setup().expect("Runtime setup failed.");

        this
    }

    fn setup(&mut self) -> Result<(), RuntimeError> {
        // let scope = self.root_scope.clone();

        let mut module_exporter = HashMap::new();
        module_exporter.insert("std".to_string(), stdlib::std().to_module_item());
        self.modules = module_exporter;

        for path in stdlib::auto_use() {
            let temp: Vec<String> = path
                .split("::")
                .into_iter()
                .map(|v| v.to_string())
                .collect();
            self.namespace_use
                .insert(temp.last().unwrap().to_string(), temp);
        }

        Ok(())
    }

    pub fn trace(&self) {
        println!("{:#?}", self.scopes);
    }

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
            // let root_scope = self.root_scope.clone();
            let new_scope = self.set_var(
                &name,
                Value::Function(types::FunctionType::DScript(func.clone())),
            )?;

            Ok((
                Some(new_scope),
                Value::Function(types::FunctionType::DScript(func)),
            ))
        } else {
            Ok((None, Value::Function(types::FunctionType::DScript(func))))
        }
    }

    pub fn execute(&mut self, code: &str) -> Result<Value, Error> {
        let ast = DioscriptAst::from_string(code)?;
        Ok(self.execute_ast(ast)?)
    }

    pub fn execute_ast(&mut self, ast: DioscriptAst) -> Result<Value, RuntimeError> {
        let result = self.execute_scope(ast.stats)?;
        Ok(result)
    }

    fn enter_scope(&mut self, i: bool) {
        let scope = if i { Scope::fun() } else { Scope::gen() };
        self.scopes.push(scope);
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn execute_scope(&mut self, statements: Vec<DioAstStatement>) -> Result<Value, RuntimeError> {
        let mut result: Value = Value::None;
        let mut finish = false;
        self.enter_scope(false);
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
                    // let name = var.0.clone();
                    // let value = var.1.clone();
                    let name = var.name.clone();
                    let value = var.expr.clone();
                    let value = self.execute_calculate(value)?;
                    let _scope = self.set_var(&name, value)?;
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
                        } else {
                            if let Some(otherwise) = otherwise {
                                result = self.execute_scope(otherwise)?;
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
                            let iter = self.to_value(iter)?;
                            if iter.value_name() == "list" {
                                for i in iter.as_list().unwrap() {
                                    self.set_var(&var, i.clone())?;
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
                _ => {}
            }
        }
        self.leave_scope();
        Ok(result)
    }

    fn to_value(&mut self, value: AstValue) -> Result<Value, RuntimeError> {
        match value {
            AstValue::None => Ok(Value::None),
            AstValue::String(v) => Ok(Value::String(v)),
            AstValue::Number(v) => Ok(Value::Number(v)),
            AstValue::Boolean(v) => Ok(Value::Boolean(v)),
            AstValue::List(v) => {
                let mut res = Vec::new();
                for i in v {
                    let value = self.to_value(i)?;
                    res.push(value);
                }
                Ok(Value::List(res))
            }
            AstValue::Dict(v) => {
                let mut res = HashMap::new();
                for (k, v) in v {
                    res.insert(k, self.to_value(v)?);
                }
                Ok(Value::Dict(res))
            }
            AstValue::Tuple((a, b)) => {
                let a = self.to_value(*a)?;
                let b = self.to_value(*b)?;
                Ok(Value::Tuple((Box::new(a), Box::new(b))))
            }
            AstValue::Element(e) => {
                let element = self.to_element(e)?;
                Ok(Value::Element(element))
            }
            AstValue::Variable(n) => {
                let value = self.get_var(&n)?.1;
                Ok(value)
            }
            AstValue::VariableIndex((n, i)) => {
                let value = self.to_value(AstValue::Variable(n))?;
                let index = self.to_value(*i)?;
                let data = self.get_from_index(value, index)?;
                Ok(data)
            }
            AstValue::FunctionCaller(caller) => {
                let data = self.execute_function(caller)?;
                Ok(data)
            }
            AstValue::FunctionDefine(define) => {
                Ok(Value::Function(types::FunctionType::DScript(define)))
            }
        }
    }

    fn deref_value(&self, value: Value) -> Result<Value, RuntimeError> {
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
                let data = self
                    .data
                    .get(&id)
                    .ok_or(RuntimeError::PoniterDataNotFound {
                        name: id.to_string(),
                    })?;
                #[allow(unreachable_patterns)]
                match data {
                    DataType::Variable(v) => Ok(v.clone()),
                    _ => Err(RuntimeError::PoniterDataNotFound {
                        name: id.to_string(),
                    }),
                }
            }
            _ => Ok(value),
        }
    }

    fn execute_function(&mut self, caller: FunctionCall) -> Result<Value, RuntimeError> {
        let name = caller.name;
        let params = caller.arguments;
        let mut par = vec![];
        for i in params {
            let v = self.to_value(i)?;
            par.push(v);
        }

        let func = self.get_function(name)?;

        match func {
            types::FunctionType::DScript(f) => {
                let f = f.clone();
                match &f.params {
                    dioscript_parser::ast::ParamsType::Variable(v) => {
                        self.set_var(&v, Value::List(par))?;
                    }
                    dioscript_parser::ast::ParamsType::List(v) => {
                        if v.len() != par.len() {
                            return Err(RuntimeError::IllegalArgumentsNumber {
                                need: v.len() as i16,
                                provided: par.len() as i16,
                            });
                        }
                        for (i, v) in v.iter().enumerate() {
                            self.set_var(v, par.get(i).unwrap().clone())?;
                        }
                    }
                }
                let result = self.execute_scope(f.inner)?;
                return Ok(result);
            }
            types::FunctionType::Rusty((f, need_param_num)) => {
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

    fn execute_function_by_ft(
        &mut self,
        func: FunctionType,
        par: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        match func {
            types::FunctionType::DScript(f) => {
                let f = f.clone();
                match &f.params {
                    dioscript_parser::ast::ParamsType::Variable(v) => {
                        self.set_var(&v, Value::List(par))?;
                    }
                    dioscript_parser::ast::ParamsType::List(v) => {
                        if v.len() != par.len() {
                            return Err(RuntimeError::IllegalArgumentsNumber {
                                need: v.len() as i16,
                                provided: par.len() as i16,
                            });
                        }
                        for (i, v) in v.iter().enumerate() {
                            self.set_var(v, par.get(i).unwrap().clone())?;
                        }
                    }
                }
                let result = self.execute_scope(f.inner)?;
                return Ok(result);
            }
            types::FunctionType::Rusty((f, need_param_num)) => {
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

    fn get_function(&self, name: FunctionName) -> Result<FunctionType, RuntimeError> {
        match name {
            FunctionName::Single(name) => {
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

    fn get_module_value(&self, mut namespace: Vec<String>) -> Result<ModuleItem, RuntimeError> {
        let data = self.load_from_module(namespace.clone());
        match data {
            Ok(v) => {
                return Ok(v);
            }
            Err(_) => {
                let v = self.namespace_use.get(&namespace[0]);
                if let Some(used) = v {
                    if used.last().unwrap() == &namespace[0] {
                        namespace.remove(0);
                    }
                    let module_path = used.iter().chain(namespace.iter()).cloned().collect();
                    let v = self.load_from_module(module_path)?;
                    return Ok(v);
                } else {
                    return Err(RuntimeError::ModuleNotFound {
                        module: namespace[0].to_string(),
                    });
                }
            }
        }
    }

    fn load_from_module(&self, namespace: Vec<String>) -> Result<ModuleItem, RuntimeError> {
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

    fn execute_calculate(&mut self, expr: CalcExpr) -> Result<Value, RuntimeError> {
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
            CalcExpr::Mod(_, _) => Ok(Value::Boolean(false)),
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
            },
        }
    }

    fn execute_link_expr(&mut self, v: LinkExpr) -> Result<Value, RuntimeError> {
        let mut this = self.to_value(v.this)?;
        let list = v.list;
        for op in list {
            match op {
                dioscript_parser::parser::LinkExprPart::Field(field) => {
                    this = self.deref_value(this.clone())?;
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
                dioscript_parser::parser::LinkExprPart::FunctionCall(call) => match &this {
                    Value::String(content) => {
                        let mut pararms = vec![Value::String(content.to_string())];
                        for i in call.arguments {
                            let v = self.to_value(i)?;
                            pararms.push(v);
                        }
                        let v = self.get_var("string")?.1;
                        let v = self.deref_value(v)?;
                        if let Value::Dict(v) = v {
                            if let Some(Value::Function(f)) = v.get(&call.name.as_single()) {
                                this = self.execute_function_by_ft(f.clone(), pararms)?;
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
                },
            }
        }
        Ok(self.deref_value(this)?)
    }

    fn get_var(&self, name: &str) -> Result<(Uuid, Value), RuntimeError> {
        for scope in self.scopes.iter().rev() {
            if scope.isolate {
                break;
            }
            if let Some(uuid) = scope.data.get(name) {
                if let Some(data_type) = self.data.get(uuid) {
                    let value = data_type.as_variable().unwrap();
                    return Ok((uuid.clone(), value));
                }
                break;
            }
        }
        Err(RuntimeError::VariableNotFound {
            name: name.to_string(),
        })
    }

    fn set_var(&mut self, name: &str, value: Value) -> Result<Uuid, RuntimeError> {
        // let value = match value {
        //     Value::List(list) => {
        //         let mut result = vec![];
        //         for (i, v) in list.iter().enumerate() {
        //             if let Value::Reference(_) = v {
        //                 // ignore
        //                 result.push(v.clone());
        //             } else {
        //                 let name = format!("{name}[{i}]");
        //                 let id = self.set_var(&name, v.clone())?;
        //                 result.push(Value::Reference(id));
        //             }
        //         }
        //         Value::List(result)
        //     }
        //     Value::Dict(dict) => {
        //         let mut result = HashMap::new();
        //         for (k, v) in dict {
        //             if let Value::Reference(_) = v {
        //                 // ignore
        //                 result.insert(k, v);
        //             } else {
        //                 let name = format!("{name}[{k}]");
        //                 let id = self.set_var(&name, v.clone())?;
        //                 result.insert(k, Value::Reference(id));
        //             }
        //         }
        //         Value::Dict(result)
        //     }
        //     Value::Tuple(tuple) => {
        //         let first = {
        //             if let Value::Reference(_) = *tuple.0.clone() {
        //                 // ignore
        //                 Box::new(*tuple.0)
        //             } else {
        //                 let name = format!(".{name}.0");
        //                 let id = self.set_var(&name, *tuple.0)?;
        //                 Box::new(Value::Reference(id))
        //             }
        //         };
        //         let second = {
        //             if let Value::Reference(_) = *tuple.1.clone() {
        //                 // ignore
        //                 Box::new(*tuple.1)
        //             } else {
        //                 let name = format!(".{name}.1");
        //                 let id = self.set_var(&name, *tuple.1)?;
        //                 Box::new(Value::Reference(id))
        //             }
        //         };
        //         Value::Tuple((first, second))
        //     }
        //     _ => value,
        // };

        let id = if let Ok((id, _)) = self.get_var(name) {
            let data = self.data.get_mut(&id).unwrap();
            #[allow(unreachable_patterns)]
            match data {
                DataType::Variable(v) => {
                    *v = value;
                }
                _ => (),
            }
            id
        } else {
            let id = Uuid::new_v4();
            let _ = self
                .data
                .insert(id, DataType::Variable(value));
            if let Some(current_scope) = self.scopes.last_mut() {
                current_scope.data.insert(name.to_string(), id);
            }
            id
        };
        return Ok(id);
    }

    #[allow(dead_code)]
    fn create_data(&mut self, data: Value) -> Result<Uuid, RuntimeError> {
        let id = Uuid::new_v4();
        self.data.insert(id, DataType::Variable(
            data
        ));
        Ok(id)
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

    fn to_element(&mut self, element: AstElement) -> Result<Element, RuntimeError> {
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
                        } else {
                            if let Some(otherwise) = v.otherwise {
                                temp = self.execute_scope(otherwise)?;
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
                            let iter = self.to_value(iter)?;
                            if iter.value_name() == "list" {
                                for i in iter.as_list().unwrap() {
                                    self.set_var(&var, i.clone())?;
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

#[derive(Debug)]
pub struct Scope {
    isolate: bool,
    data: HashMap<String, Uuid>,
}

impl Scope {
    pub fn gen() -> Self {
        Self {
            isolate: false,
            data: HashMap::new(),
        }
    }

    pub fn fun() -> Self {
        Self {
            isolate: true,
            data: HashMap::new(),
        }
    }
}

pub enum DataType {
    Variable(Value),
}

impl DataType {
    pub fn as_variable(&self) -> Option<Value> {
        #[allow(unreachable_patterns)]
        match self {
            Self::Variable(r) => return Some(r.clone()),
            _ => (),
        }
        None
    }
}

