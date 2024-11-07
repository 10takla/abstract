use ast::{expand::expand, name_resolve::name_resolve};
use lexer::{items::Items, Code, Parse};

pub mod ast;
pub mod lexer;
pub mod parser;

pub fn compile(code: &'static str) -> Option<()> {
    let items = Items::parse(&Code::new(code))?;
    println!("{}\n", items);
    let (refs, _) = name_resolve(&items, None);
    println!("{}", expand(&items, &refs));
    Some(())
}
