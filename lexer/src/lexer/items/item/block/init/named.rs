use super::unnamed::{UnnamedBlock, UnnamedBlockDiag};
use crate::{
    items::item::ident::IdentDiag,
    lexer::{
        check,
        items::item::{ident::Ident, Item},
        Code, DiagParse, Slicable,
    },
};
use macros::Parse;
use std::{
    fmt::{Debug, Display},
    ops::RangeInclusive,
};

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
#[grammar(
    Ident UnnamedBlock
)]
#[diag(NamedBlockDiag)]
pub struct NamedBlock<'s> {
    #[diag(Name)]
    pub name: Ident<'s>,
    #[diag(UnnamedBlock)]
    pub block: UnnamedBlock<'s>,
}

#[derive(PartialEq, Debug)]
pub enum NamedBlockDiag {
    Name(IdentDiag),
    UnnamedBlock(UnnamedBlockDiag),
}

impl<'s> NamedBlock<'s> {
    pub fn new(
        idnet_slice: RangeInclusive<usize>,
        (items, brackets): (Vec<Item<'s>>, [usize; 2]),
        code: &Code<'s>,
    ) -> Self {
        Self {
            name: Ident::new(idnet_slice, code),
            block: UnnamedBlock::new(items, brackets),
        }
    }
}

impl Display for NamedBlock<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NamedBlock({} => {})", self.name, self.block.items)
    }
}

#[test]
fn parse() {
    check("main {}", |code| NamedBlock {
        name: Ident::new(0..=3, code),
        block: UnnamedBlock::new(vec![], [5, 6]),
    });
    check("  main   {  }  ", |code| NamedBlock {
        name: Ident::new(2..=5, code),
        block: UnnamedBlock::new(vec![], [9, 12]),
    });
    check("  main   { dsffds }  ", |code| NamedBlock {
        name: Ident::new(2..=5, code),
        block: UnnamedBlock::new(vec![Item::Ident(Ident::new(11..=16, code))], [9, 18]),
    });
}
