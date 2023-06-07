use dioscript_parser::types::Value;

pub type Function = Box<dyn Fn(Vec<Value>) -> Value>;

pub fn element_to_html() -> Function {
    Box::new(|v| {
        let element = v.get(0).unwrap();
        let element = element.as_element().unwrap();
        Value::String(element.to_html())
    })
}
