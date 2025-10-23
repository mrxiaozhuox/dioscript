use std::{
    fs::{create_dir_all, read_to_string},
    path::PathBuf,
};

use anyhow::anyhow;
use dioscript_runtime::Value;

use crate::BuildArgs;

pub fn build(args: &BuildArgs) -> anyhow::Result<String> {
    let target = &args.target;
    let file_name = &args.file;
    let out_dir = &args.out_dir;
    let template = &args.template;

    let build_target = BuildTarget::from_str(target);
    let file_path = PathBuf::from(file_name);
    let file_content = read_to_string(&file_path)?;
    let file_stem = file_path.file_stem().unwrap().to_str().unwrap();

    let template = if let Some(v) = template {
        let file = PathBuf::from(v);
        if !file.is_file() {
            "<dioscript />".to_string()
        } else {
            read_to_string(file)?
        }
    } else {
        "<dioscript />".to_string()
    };

    match build_target {
        BuildTarget::HtmlStatic => {
            let ast = dioscript_parser::ast::DioscriptAst::from_string(&file_content)?;
            let mut runtime = dioscript_runtime::Executor::init();
            let result = runtime.execute(ast)?;
            if let Value::Element(e) = result {
                let html = template.replace("<dioscript />", &e.to_html());
                if !PathBuf::from(out_dir).is_dir() {
                    create_dir_all(out_dir)?;
                }
                std::fs::write(format!("{}/{}.html", out_dir, file_stem), html)?;
                Ok(format!("{}/{}.html", out_dir, file_stem))
            } else {
                Err(anyhow!("result data type is not Element"))
            }
        }
        BuildTarget::Unknown => Err(anyhow!("dioscript not support `{target}` builder.")),
    }
}

pub enum BuildTarget {
    HtmlStatic,
    Unknown,
}

impl BuildTarget {
    pub fn from_str(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "html@static" => Self::HtmlStatic,
            _ => Self::Unknown,
        }
    }
}
