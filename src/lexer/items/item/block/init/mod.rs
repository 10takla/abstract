pub mod named;
pub mod unnamed;

use crate::lexer::check;
use macros::Parse;
use named::NamedBlock;
use unnamed::UnnamedBlock;

#[derive(PartialEq, Debug, Parse)]
pub enum InitBlock {
    Named(NamedBlock),
    Unnamed(UnnamedBlock),
}

#[test]
pub fn parse_init_block() {
    check("main {}", |code| {
        InitBlock::Named(NamedBlock::new([0, 3], (vec![], [5, 6]), code))
    });
    check(" {  } ", |_| {
        InitBlock::Unnamed(UnnamedBlock::new(vec![], [1, 4]))
    });
}
