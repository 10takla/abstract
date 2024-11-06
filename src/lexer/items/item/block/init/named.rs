use super::unnamed::UnnamedBlock;
use crate::lexer::{
    check,
    items::{
        item::{ident::Ident, Item},
        shared::whitespaces::Whitespaces,
    },
    Code, Parse, Slicable,
};
use macros::Parse;
use std::fmt::{Debug, Display};

#[derive(PartialEq, Debug, Parse)]
#[grammar(
    Ident UnnamedBlock
)]
pub struct NamedBlock {
    pub name: Ident,
    pub block: UnnamedBlock,
}

impl NamedBlock {
    pub fn new(
        idnet_slice: [usize; 2],
        (items, brackets): (Vec<Item>, [usize; 2]),
        code: &Code,
    ) -> Self {
        Self {
            name: Ident::new(idnet_slice, code),
            block: UnnamedBlock::new(items, brackets),
        }
    }
}

impl Display for NamedBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NamedBlock({})", self.block.items)
    }
}

#[test]
fn parse_named() {
    check("main {}", |code| NamedBlock {
        name: Ident::new([0, 3], code),
        block: UnnamedBlock::new(vec![], [5, 6]),
    });
    check("  main   {  }  ", |code| NamedBlock {
        name: Ident::new([2, 5], code),
        block: UnnamedBlock::new(vec![], [9, 12]),
    });
    check("  main   { dsffds }  ", |code| NamedBlock {
        name: Ident::new([2, 5], code),
        block: UnnamedBlock::new(vec![Item::Ident(Ident::new([11, 16], code))], [9, 18]),
    });
}
