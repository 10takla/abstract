pub mod named;
pub mod unnamed;

use macros::Parse;
use named::{NamedBlock, NamedBlockDiag};
use unnamed::{UnnamedBlock, UnnamedBlockDiag};
use crate::lexer::check;

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
#[diag(InitBlockDiag)]
pub enum InitBlock<'s> {
    Named(NamedBlock<'s>),
    Unnamed(UnnamedBlock<'s>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum InitBlockDiag {
    Named(NamedBlockDiag),
    Unnamed(UnnamedBlockDiag),
}

#[test]
pub fn parse() {
    check("main {}", |code| {
        InitBlock::Named(NamedBlock::new(0..=3, (vec![], [5, 6]), code))
    });
    check(" {  } ", |_| {
        InitBlock::Unnamed(UnnamedBlock::new(vec![], [1, 4]))
    });
}
