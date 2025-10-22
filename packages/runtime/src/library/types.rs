pub mod string {
    use crate::{error::RuntimeError, module::ModuleGenerator, types::Value, Runtime};

    pub fn join(_rt: &mut Runtime, mut args: Vec<Value>) -> Result<Value, RuntimeError> {
        let this = args.first().unwrap().as_string().unwrap();
        let mut result = this;
        args.remove(0);
        for i in args {
            result.push_str(&i.to_string());
        }
        Ok(Value::String(result))
    }

    pub fn len(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let this = args.first().unwrap().as_string().unwrap();
        Ok(Value::Number(this.len() as f64))
    }

    pub fn repeat(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let this = args.first().unwrap().as_string().unwrap();
        let number = args.get(1).unwrap().as_number().unwrap_or(1.0);
        Ok(Value::String(this.repeat(number as usize)))
    }

    pub fn to_bytes(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let this = args.first().unwrap().as_string().unwrap();
        let v = this
            .into_bytes()
            .iter()
            .map(|v| {
                let v = *v as f64;
                Value::Number(v)
            })
            .collect();
        Ok(Value::List(v))
    }

    pub fn is_empty(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let this = args.first().unwrap().as_string().unwrap();
        Ok(Value::Boolean(this.is_empty()))
    }

    pub fn lowercase(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let this = args.first().unwrap().as_string().unwrap();
        Ok(Value::String(this.to_lowercase()))
    }

    pub fn uppercase(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let this = args.first().unwrap().as_string().unwrap();
        Ok(Value::String(this.to_uppercase()))
    }

    pub fn split(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let this = args.first().unwrap().as_string().unwrap();
        let sep = args.get(1).unwrap().as_string().unwrap();
        let result = this
            .split(&sep)
            .map(|v| Value::String(v.to_string()))
            .collect::<Vec<Value>>();
        Ok(Value::List(result))
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

        module.insert_rusty_function("to_bytes", to_bytes, 1);

        module
    }
}

pub mod number {

    use core::f64;

    use crate::{error::RuntimeError, module::ModuleGenerator, types::Value, Runtime};

    pub fn abs(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let num = args.first().unwrap().as_number().unwrap();
        Ok(Value::Number(num.abs()))
    }

    pub fn max(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let mut max = f64::MIN;
        for arg in args {
            if let Value::Number(num) = arg {
                if num > max {
                    max = num;
                }
            }
        }
        Ok(Value::Number(max))
    }

    pub fn min(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let mut min = f64::MAX;
        for arg in args {
            if let Value::Number(num) = arg {
                if num < min {
                    min = num;
                }
            }
        }
        Ok(Value::Number(min))
    }

    pub fn export() -> ModuleGenerator {
        let mut module = ModuleGenerator::new();

        module.insert_rusty_function("abs", abs, 1);
        module.insert_rusty_function("max", max, -1);
        module.insert_rusty_function("min", min, -1);

        module
    }
}
