use dioscript;
fn main() {
    let ast = dioscript::parser::parse_rsx("@a = 1; @b = 2;");
    println!("{ast:?}");
}
