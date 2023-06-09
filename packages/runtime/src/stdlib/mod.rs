use crate::function::Function;
use std::collections::HashMap;

pub type Exporter = (Vec<String>, HashMap<String, (Function, i16)>);

pub mod value {
    use dioscript_parser::types::Value;

    use crate::{function::Function, Runtime};
    use std::collections::HashMap;

    use super::Exporter;

    pub fn type_name(_: &mut Runtime, args: Vec<Value>) -> Value {
        let value = args.get(0).unwrap();
        return Value::String(value.value_name());
    }

    pub fn export() -> Exporter {
        let map: HashMap<String, (Function, i16)> = HashMap::new();
        (vec!["std".to_string(), "value".to_string()], map)
    }
}

pub mod console {
    use std::collections::HashMap;

    use dioscript_parser::types::Value;

    use crate::{function::Function, Runtime};

    use super::Exporter;

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

    pub fn export() -> Exporter {
        let map: HashMap<String, (Function, i16)> = HashMap::new();
        (vec!["std".to_string(), "value".to_string()], map)
    }
}

pub mod runtime {

    use dioscript_parser::types::Value;

    use crate::{function::Function, Runtime};
    use std::collections::HashMap;

    use super::Exporter;

    pub fn execute(rt: &mut Runtime, args: Vec<Value>) -> Value {
        let value = args.get(0).unwrap();
        if let Value::String(v) = value {
            let result = rt.execute(&v).unwrap();
            return result;
        }
        Value::None
    }

    pub fn export() -> Exporter {
        let map: HashMap<String, (Function, i16)> = HashMap::new();
        (vec!["std".to_string(), "runtime".to_string()], map)
    }
}

pub fn root_export() -> Exporter {
    let mut map: HashMap<String, (Function, i16)> = HashMap::new();
    map.insert("type".to_string(), (value::type_name, 1));

    map.insert("print".to_string(), (console::print, -1));
    map.insert("println".to_string(), (console::println, -1));

    map.insert("execute".to_string(), (runtime::execute, -1));
    (vec![], map)
}
