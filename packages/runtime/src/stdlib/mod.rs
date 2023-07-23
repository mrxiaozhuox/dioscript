use std::collections::HashMap;

use crate::types::Value;

pub mod root {

    use std::collections::HashMap;

    use crate::{function::MethodBinder, types::Value, Runtime};

    pub fn print(_: &mut Runtime, args: Vec<Value>) -> Value {
        print!("{}", iterable_to_str(args));
        return Value::None;
    }

    pub fn println(_: &mut Runtime, args: Vec<Value>) -> Value {
        println!("{}", iterable_to_str(args));
        return Value::None;
    }

    fn iterable_to_str<I, D>(iterable: I) -> String
    where
        I: IntoIterator<Item = D>,
        D: ToString,
    {
        let mut iterator = iterable.into_iter();

        let head = match iterator.next() {
            None => return String::new(),
            Some(x) => format!("{}", x.to_string()),
        };
        let body = iterator.fold(head, |a, v| format!("{}, {}", a, v.to_string()));
        format!("{}", body)
    }

    pub fn type_name(_: &mut Runtime, args: Vec<Value>) -> Value {
        let name = args.get(0).unwrap().value_name();
        return Value::String(name);
    }

    pub fn execute(rt: &mut Runtime, args: Vec<Value>) -> Value {
        let value = args.get(0).unwrap();
        if let Value::String(v) = value {
            let result = rt.execute(&v).unwrap();
            return result;
        }
        Value::None
    }

    pub fn import(_rt: &mut Runtime, args: Vec<Value>) -> Value {
        let name = args.get(0).unwrap();
        if let Value::String(name) = name {
            match name.as_str() {
                _ => {}
            }
        }
        Value::None
    }

    pub fn export() -> (
        crate::function::BindTarget,
        HashMap<std::string::String, Value>,
    ) {
        let mut exporter = MethodBinder::new(crate::function::BindTarget::Root);

        exporter.insert("print", (print, -1));
        exporter.insert("println", (println, -1));

        exporter.insert("type", (type_name, 1));

        exporter.insert("execute", (execute, -1));
        exporter.insert("import", (import, 1));
        exporter.collect()
    }
}

mod string {
    use std::collections::HashMap;

    use crate::{function::MethodBinder, types::Value, Runtime};

    pub fn join(_rt: &mut Runtime, mut args: Vec<Value>) -> Value {
        let this = args.get(0).unwrap().as_string().unwrap();
        let mut result = this;
        args.remove(0);
        for i in args {
            result.push_str(&i.to_string());
        }
        Value::String(result)
    }

    pub fn len(_rt: &mut Runtime, args: Vec<Value>) -> Value {
        let this = args.get(0).unwrap().as_string().unwrap();
        Value::Number(this.len() as f64)
    }

    pub fn repeat(_rt: &mut Runtime, args: Vec<Value>) -> Value {
        let this = args.get(0).unwrap().as_string().unwrap();
        let number = args.get(1).unwrap().as_number().unwrap_or(1.0);
        Value::String(this.repeat(number as usize))
    }

    pub fn is_empty(_rt: &mut Runtime, args: Vec<Value>) -> Value {
        let this = args.get(0).unwrap().as_string().unwrap();
        Value::Boolean(this.is_empty())
    }

    pub fn lowercase(_rt: &mut Runtime, args: Vec<Value>) -> Value {
        let this = args.get(0).unwrap().as_string().unwrap();
        Value::String(this.to_lowercase())
    }

    pub fn uppercase(_rt: &mut Runtime, args: Vec<Value>) -> Value {
        let this = args.get(0).unwrap().as_string().unwrap();
        Value::String(this.to_uppercase())
    }

    pub fn split(_rt: &mut Runtime, args: Vec<Value>) -> Value {
        let this = args.get(0).unwrap().as_string().unwrap();
        let sep = args.get(1).unwrap().as_string().unwrap();
        let result = this
            .split(&sep)
            .map(|v| Value::String(v.to_string()))
            .collect::<Vec<Value>>();
        Value::List(result)
    }

    pub fn export() -> (
        crate::function::BindTarget,
        HashMap<std::string::String, Value>,
    ) {
        let mut exporter = MethodBinder::new(crate::function::BindTarget::String);

        exporter.insert("join", (join, -1));
        exporter.insert("len", (len, 1));
        exporter.insert("repeat", (repeat, 2));

        exporter.insert("is_empty", (is_empty, 1));

        exporter.insert("lowercase", (lowercase, 1));
        exporter.insert("uppercase", (uppercase, 1));

        exporter.insert("split", (split, 2));

        exporter.collect()
    }
}

mod number {
    use std::collections::HashMap;

    use crate::{function::MethodBinder, types::Value};

    pub fn export() -> (
        crate::function::BindTarget,
        HashMap<std::string::String, Value>,
    ) {
        let mut exporter = MethodBinder::new(crate::function::BindTarget::Number);
        exporter.collect()
    }
}

mod boolean {
    use std::collections::HashMap;

    use crate::{function::MethodBinder, types::Value};

    pub fn export() -> (
        crate::function::BindTarget,
        HashMap<std::string::String, Value>,
    ) {
        let mut exporter = MethodBinder::new(crate::function::BindTarget::Boolean);
        exporter.collect()
    }
}

pub fn all() -> Vec<(crate::function::BindTarget, HashMap<String, Value>)> {
    let mut result = vec![];
    result.push(root::export());
    result.push(string::export());
    result
}
