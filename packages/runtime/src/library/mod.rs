use crate::{error::RuntimeError, module::ModuleGenerator, types::Value, Runtime};

pub mod types;

pub fn print(_: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
    print!("{}", iterable_to_str(args));
    Ok(Value::None)
}

pub fn println(_: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
    println!("{}", iterable_to_str(args));
    Ok(Value::None)
}

fn iterable_to_str<I, D>(iterable: I) -> String
where
    I: IntoIterator<Item = D>,
    D: ToString,
{
    let mut iterator = iterable.into_iter();

    let head = match iterator.next() {
        None => return String::new(),
        Some(x) => x.to_string(),
    };
    let body = iterator.fold(head, |a, v| format!("{}, {}", a, v.to_string()));
    body.to_string()
}

pub fn type_name(_: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = args.first().unwrap().value_name();
    Ok(Value::String(name))
}

pub fn execute(rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let value = args.first().unwrap();
    if let Value::String(v) = value {
        return match rt.execute(v) {
            Ok(result) => Ok(result),
            Err(err) => match err {
                crate::error::Error::Runtime(runtime_error) => Err(runtime_error),
                crate::error::Error::Parse(parse_error) => Err(RuntimeError::DynamicParseFailed {
                    err: parse_error.to_string(),
                }),
            },
        };
    }
    Ok(Value::None)
}

pub fn range(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match args.len() {
        1 => {
            let size = args.first().unwrap();
            if let Value::Number(n) = size {
                let v: Vec<Value> = (0..n.round() as usize)
                    .map(|i| Value::Number(i as f64))
                    .collect();
                return Ok(Value::List(v));
            }
            Ok(Value::None)
        }
        2 => {
            let start = args.first().unwrap();
            let end = args.get(1).unwrap();
            if let Value::Number(s) = start {
                if let Value::Number(e) = end {
                    let start_int = s.round() as usize;
                    let end_int = e.round() as usize;

                    let v: Vec<Value> = (start_int..end_int)
                        .map(|i| Value::Number(i as f64))
                        .collect();
                    return Ok(Value::List(v));
                }
            }
            Ok(Value::None)
        }
        3 => {
            let start = args.first().unwrap();
            let end = args.get(1).unwrap();
            let step = args.get(2).unwrap();

            if let Value::Number(s) = start {
                if let Value::Number(e) = end {
                    if let Value::Number(step_val) = step {
                        let start_float = *s;
                        let end_float = *e;
                        let step_float = *step_val;

                        if step_float <= 0.0 {
                            return Ok(Value::None);
                        }

                        let count = ((end_float - start_float) / step_float).ceil() as usize;

                        let v: Vec<Value> = (0..count)
                            .map(|i| Value::Number(start_float + (i as f64) * step_float))
                            .collect();

                        return Ok(Value::List(v));
                    }
                }
            }
            Ok(Value::None)
        }
        _ => Ok(Value::None),
    }
}

pub fn built_in() -> ModuleGenerator {
    let mut module_exporter = ModuleGenerator::new();

    module_exporter.insert_rusty_function("print", print, -1);
    module_exporter.insert_rusty_function("println", println, -1);
    module_exporter.insert_rusty_function("type", type_name, -1);
    module_exporter.insert_rusty_function("execute", execute, -1);
    module_exporter.insert_rusty_function("range", range, -1);

    module_exporter.insert_sub_module("string", types::string::export());
    module_exporter.insert_sub_module("number", types::number::export());

    module_exporter
}
