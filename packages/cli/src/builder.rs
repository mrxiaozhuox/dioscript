use std::{fs::read_to_string, path::PathBuf};

pub fn build(file_name: String, target: Option<String>) {
    let target = BuildTarget::from_option_str(target);
    let file_path = PathBuf::from(file_name);
    if !file_path.is_file() {
        log::error!("file not found.");
        return;
    }
    match target {
        BuildTarget::Html => {
            let ast = dioscript_parser::ast::DioscriptAst::from_string(
                &read_to_string(file_path).unwrap(),
            );
            match ast {
                Ok(ast) => {
                    let mut runtime = dioscript_runtime::Runtime::new();
                    let result = runtime.execute_ast(ast).expect("runtime failed.");
                    if let dioscript_parser::types::Value::Element(e) = result {
                        let html = "@{dioscript}".replace("@{dioscript}", &e.to_html());
                        std::fs::write("./examples/output.html", html).expect("write file failed.");
                        println!("Done.");
                    }
                }
                Err(err) => {
                    println!("{}", err.to_string());
                }
            }
        }
        BuildTarget::Wasm => todo!(),
        BuildTarget::JavaScript => todo!(),
    }
}

pub enum BuildTarget {
    Html,
    Wasm,
    JavaScript,
}

impl BuildTarget {
    pub fn from_option_str(name: Option<String>) -> Self {
        if name.is_none() {
            return Self::Html;
        }
        let name = name.unwrap();
        match name.to_lowercase().as_str() {
            "html" => Self::Html,
            "wasm" => Self::Wasm,
            "javascript" | "js" => Self::JavaScript,
            _ => Self::Html,
        }
    }
}
