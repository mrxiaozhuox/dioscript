use crate::{
    core::{
        error::RuntimeError,
        module::{OutputType, RustyExecutor},
    },
    Value,
};

pub fn print(mut rt: RustyExecutor, args: Vec<Value>) -> Result<Value, RuntimeError> {
    for i in args {
        rt.output(OutputType::Print, i);
    }
    Ok(Value::None)
}
