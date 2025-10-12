use crate::module::ModuleGenerator;

pub mod root {

    use crate::{module::ModuleGenerator, types::Value, Runtime};

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
            return match rt.execute(&v) {
                Ok(result) => result,
                Err(err) => Value::Tuple((
                    Box::from(Value::String("error".to_string())),
                    Box::from(Value::String(err.to_string())),
                )),
            };
        }
        Value::None
    }

    pub fn export() -> ModuleGenerator {
        let mut module = ModuleGenerator::new();

        module.insert_rusty_function("print", print, -1);
        module.insert_rusty_function("println", println, -1);
        module.insert_rusty_function("type", type_name, 1);
        module.insert_rusty_function("execute", execute, -1);

        return module;
    }
}

mod string {
    use crate::{module::ModuleGenerator, types::Value, Runtime};

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

    pub fn export() -> ModuleGenerator {
        let mut module = ModuleGenerator::new();

        module.insert_rusty_function("join", join, -1);
        module.insert_rusty_function("len", len, 1);
        module.insert_rusty_function("repeat", repeat, 2);

        module.insert_rusty_function("is_empty", is_empty, 1);

        module.insert_rusty_function("lowercase", lowercase, 1);
        module.insert_rusty_function("uppercase", uppercase, 1);

        module.insert_rusty_function("split", split, 2);

        module
    }
}

mod number {

    use core::f64;

    use crate::{module::ModuleGenerator, types::Value, Runtime};

    pub fn abs(_rt: &mut Runtime, args: Vec<Value>) -> Value {
        let num = args.first().unwrap().as_number().unwrap();
        Value::Number(num.abs())
    }

    pub fn max(_rt: &mut Runtime, args: Vec<Value>) -> Value {
        let mut max = f64::MIN;
        for arg in args {
            if let Value::Number(num) = arg {
                if num > max {
                    max = num;
                }
            }
        }
        Value::Number(max)
    }

    pub fn min(_rt: &mut Runtime, args: Vec<Value>) -> Value {
        let mut min = f64::MAX;
        for arg in args {
            if let Value::Number(num) = arg {
                if num < min {
                    min = num;
                }
            }
        }
        Value::Number(min)
    }

    pub fn export() -> ModuleGenerator {
        let mut module = ModuleGenerator::new();

        module.insert_rusty_function("abs", abs, 1);
        module.insert_rusty_function("max", max, -1);
        module.insert_rusty_function("min", min, -1);

        module
    }
}

pub fn std() -> ModuleGenerator {
    let mut export = root::export();
    export.insert_sub_module("string", string::export());
    export.insert_sub_module("number", number::export());
    export
}

pub fn auto_use() -> Vec<String> {
    let v = ["std::print", "std::println", "std::type", "std::execute"];
    v.iter().map(|v| v.to_string()).collect()
}
