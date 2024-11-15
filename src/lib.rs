use ast::{expand::expand, name_resolve::name_resolve};
use lexer::{items::Items, parse};

pub fn compile(code: &'static str) -> Result<Items, ()> {
    let items = parse(code);
    let (refs, _) = name_resolve(&items, None);
    Ok(expand(&items, &refs))
}
