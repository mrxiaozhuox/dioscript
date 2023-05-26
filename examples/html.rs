use std::fs;

use dioscript::{self, types::Value};
fn main() {
    let ast = dioscript::ast::DioscriptAst::from_string(include_str!("../scripts/test.ds"));
    match ast {
        Ok(ast) => {
            let mut runtime = dioscript::runtime::Runtime::new();
            let result = runtime.execute_ast(ast).expect("runtime failed.");
            if let Value::Element(e) = result {
                let html = include_str!("./template.html").to_string();
                let html = html.replace("@{dioscript}", &e.to_html());
                fs::write("./examples/output.html", html).expect("write file failed.");
            }
        }
        Err(err) => {
            println!("{}", err.to_string());
        }
    }
}
