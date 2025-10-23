use std::{fs::read_to_string, path::PathBuf};

use dioscript_parser::ast::DioscriptAst;
use dioscript_runtime::{Executor, Value};

use crate::{playground::PlaygroundOutputHandler, RunArgs};

pub fn run(args: &RunArgs) -> anyhow::Result<Value> {
    let file_name = &args.file;

    let file_path = PathBuf::from(file_name);
    let file_content = read_to_string(&file_path)?;

    let ast = DioscriptAst::from_string(&file_content)?;

    let mut runtime = Executor::init();
    runtime.with_output_handler(Box::new(PlaygroundOutputHandler));

    let value = runtime.execute(ast)?;

    Ok(value)
}
