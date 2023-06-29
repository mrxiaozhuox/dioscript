pub mod root {

    use crate::{function::Exporter, Runtime, types::Value, error::RuntimeError};

    use super::math;

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

    pub fn execute(rt: &mut Runtime, args: Vec<Value>) -> Value {
        let value = args.get(0).unwrap();
        if let Value::String(v) = value {
            let result = rt.execute(&v).unwrap();
            return result;
        }
        Value::None
    }

    pub fn import(rt: &mut Runtime, args: Vec<Value>) -> Value {
        let name = args.get(0).unwrap();
        if let Value::String(name) = name {
            match name.as_str() {
                "math" => {
                    return math::export(rt).unwrap();
                }
                _ => {}
            }
        }
        Value::None
    }

    pub fn export(rt: &mut Runtime) -> Result<Value, RuntimeError> {
        let mut exporter = Exporter::new("rf");
        exporter.insert("print", (print, -1));
        exporter.insert("println", (println, -1));
        exporter.insert("execute", (execute, -1));
        exporter.insert("import", (import, 1));
        exporter.bind(rt)
    }
}

pub mod math {
    use crate::{function::Exporter, Runtime, types::Value, error::RuntimeError};

    pub fn abs(_: &mut Runtime, args: Vec<Value>) -> Value {
        let num = args.get(0).unwrap();
        if let Value::Number(num) = num {
            return Value::Number(num.abs());
        }
        Value::None
    }
    pub fn export(rt: &mut Runtime) -> Result<Value, RuntimeError> {
        let mut exporter = Exporter::new("math");
        exporter.insert("abs", (abs, 1));
        exporter.bind(rt)
    }
}
