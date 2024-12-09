use colored::Colorize;
use lexer::{
    lexer2::{ErrorType, Diag, Slice, S},
    parse,
};

fn main() {
    let source = r#"abc 43 c2 78 dd 22s " 22s"#;

    let errors = parse(source).1;

    for e in errors {
        println!("{e}");
    }
}