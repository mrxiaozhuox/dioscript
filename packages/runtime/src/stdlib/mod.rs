use crate::function::Function;
use std::collections::HashMap;

pub type Exporter = (Vec<String>, HashMap<String, (Function, i16)>);

pub mod value {
    use dioscript_parser::types::Value;

    use crate::function::Function;
    use std::collections::HashMap;

    use super::Exporter;

    pub fn type_name(args: Vec<Value>) -> Value {
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

    use crate::function::Function;

    use super::Exporter;

    pub fn print(args: Vec<Value>) -> Value {
        print!("{}", iterable_to_str(args));
        return Value::None;
    }

    pub fn println(args: Vec<Value>) -> Value {
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

pub fn root_export() -> Exporter {
    let mut map: HashMap<String, (Function, i16)> = HashMap::new();
    map.insert("type".to_string(), (Box::new(|v| value::type_name(v)), 1));

    map.insert("print".to_string(), (Box::new(|v| console::print(v)), -1));
    map.insert(
        "println".to_string(),
        (Box::new(|v| console::println(v)), -1),
    );

    (vec![], map)
}
