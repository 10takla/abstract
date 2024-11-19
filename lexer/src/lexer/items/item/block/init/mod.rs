pub mod named;
pub mod unnamed;

use macros::Parse;
use named::{NamedBlock, NamedBlockDiag};
use unnamed::{UnnamedBlock, UnnamedBlockDiag};

use crate::lexer::check;

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
#[diag(InitDiag)]
pub enum Init<'s> {
    Named(NamedBlock<'s>),
    Unnamed(UnnamedBlock<'s>),
}

#[derive(PartialEq, Debug)]
pub enum InitDiag {
    Named(NamedBlockDiag),
    Unnamed(UnnamedBlockDiag),
}

#[test]
pub fn parse() {
    check("main {}", |code| {
        Init::Named(NamedBlock::new(0..=3, (vec![], [5, 6]), code))
    });
    check(" {  } ", |_| {
        Init::Unnamed(UnnamedBlock::new(vec![], [1, 4]))
    });
}
