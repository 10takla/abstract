#![allow(unused)]

use ast::{expand::expand, name_resolve::name_resolve};
use lexer::{items::Items, Code, Parse};

pub mod ast;
pub mod lexer;
pub mod parser;

pub fn compile(code: &'static str) -> Result<Items, ()> {
    let items = Items::parse(&Code::new(code)).ok_or(())?;
    let (refs, _) = name_resolve(&items, None);
    Ok(expand(&items, &refs))
}
