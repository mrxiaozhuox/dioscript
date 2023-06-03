use std::{
    fs::{create_dir_all, read_to_string},
    path::PathBuf,
};

use anyhow::anyhow;

pub fn build(file_name: &str, target: &str, out_dir: &str) -> anyhow::Result<()> {
    let build_target = BuildTarget::from_str(&target);
    let file_path = PathBuf::from(file_name);
    let file_content = read_to_string(&file_path)?;
    let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
    match build_target {
        BuildTarget::Static => {
            let ast = dioscript_parser::ast::DioscriptAst::from_string(&file_content)?;
            let mut runtime = dioscript_runtime::Runtime::new();
            let result = runtime.execute_ast(ast)?;
            if let dioscript_parser::types::Value::Element(e) = result {
                let html = "<dioscript />".replace("<dioscript />", &e.to_html());
                if !PathBuf::from(out_dir).is_dir() {
                    create_dir_all(out_dir)?;
                }
                std::fs::write(format!("{}/{}.html", out_dir, file_stem), html)?;
            }
        }
        BuildTarget::Unknown => {
            return Err(anyhow!("dioscript not support `{target}` builder."));
        }
    }
    Ok(())
}

pub enum BuildTarget {
    Static,
    Unknown,
}

impl BuildTarget {
    pub fn from_str(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "static" => Self::Static,
            _ => Self::Unknown,
        }
    }
}
