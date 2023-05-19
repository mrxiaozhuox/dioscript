use dioscript;
fn main() {
    let ast = dioscript::ast::DioscriptAst::from_string(include_str!("../test.rsx"));
    match ast {
        Ok(ast) => {
            let mut runtime = dioscript::runtime::Runtime::new();
            let result = runtime.execute_ast(ast);
            println!("{:#?}", result);
        }
        Err(err) => {
            panic!("{}", err.to_string());
        }
    }
}
