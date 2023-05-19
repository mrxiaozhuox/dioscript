use dioscript;
fn main() {
    let ast = dioscript::ast::DioscriptAst::to_ast(include_str!("../test.rsx"));
    let mut runtime = dioscript::runtime::Runtime::new();
    let result = runtime.execute_ast(ast);
    println!("{:#?}", result);
}
