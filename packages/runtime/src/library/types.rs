pub mod string {
    use crate::{error::RuntimeError, module::ModuleGenerator, types::Value, Runtime};

    pub fn join(_rt: &mut Runtime, mut args: Vec<Value>) -> Result<Value, RuntimeError> {
        Ok(Value::String("".to_string()))
    }

    pub fn export() -> ModuleGenerator {
        let mut module = ModuleGenerator::new();

        module.insert_rusty_function("join", join, -1);

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

    // pub fn max(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
    //     let mut max = f64::MIN;
    //     for arg in args {
    //         if let Value::Number(num) = arg {
    //             if num > max {
    //                 max = num;
    //             }
    //         }
    //     }
    //     Ok(Value::Number(max))
    // }
    //
    // pub fn min(_rt: &mut Runtime, args: Vec<Value>) -> Result<Value, RuntimeError> {
    //     let mut min = f64::MAX;
    //     for arg in args {
    //         if let Value::Number(num) = arg {
    //             if num < min {
    //                 min = num;
    //             }
    //         }
    //     }
    //     Ok(Value::Number(min))
    // }

    pub fn export() -> ModuleGenerator {
        let mut module = ModuleGenerator::new();

        module.insert_rusty_function("abs", abs, 1);

        module
    }
}

pub mod list {
    use crate::{error::RuntimeError, module::ModuleGenerator, types::Value, Runtime};

    pub fn insert(rt: &mut Runtime, mut args: Vec<Value>) -> Result<Value, RuntimeError> {
        let target = args.first().unwrap().clone();
        args.remove(0);

        if let Value::Reference(refe) = target {
            let list = rt.get_ref_value(&refe)?;
            if let Value::List(mut list) = list {
                list.extend(args);
                rt.set_ref_value(&refe, Value::List(list))?;
                Ok(Value::None)
            } else {
                Err(RuntimeError::UnsupportDataType {
                    expect_type: "reference".to_string(),
                    receive_type: target.value_name(),
                })
            }
        } else {
            Err(RuntimeError::UnsupportDataType {
                expect_type: "reference".to_string(),
                receive_type: target.value_name(),
            })
        }
    }

    pub fn export() -> ModuleGenerator {
        let mut module = ModuleGenerator::new();

        module.insert_rusty_function("insert", insert, -1);

        module
    }
}
