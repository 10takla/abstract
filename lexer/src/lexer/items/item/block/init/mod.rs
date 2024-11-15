pub mod named;
pub mod unnamed;

use macros::Parse;
use named::NamedBlock;
use unnamed::UnnamedBlock;

use crate::lexer::check;

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
pub enum Init<'s> {
    Named(NamedBlock<'s>),
    Unnamed(UnnamedBlock<'s>),
}

#[test]
pub fn parse_init_block() {
    check("main {}", |code| {
        Init::Named(NamedBlock::new(0..=3, (vec![], [5, 6]), code))
    });
    check(" {  } ", |_| {
        Init::Unnamed(UnnamedBlock::new(vec![], [1, 4]))
    });
}
