use crate::{items::item::{block::init::unnamed::UnnamedBlock, ident::Ident}, lexer::{
    check, check_none, items::{item::block::init::named::NamedBlock, shared::distribution::Distribution}, Parse, Slicable
}, Slice};
use macros::Parse;
use std::fmt::Display;

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
#[grammar(
    NamedBlock Distribution
)]
pub struct InitBlockDistruct<'s> {
    pub named_block: NamedBlock<'s>,
    pub distr: Distribution<'s>,
}

impl Display for InitBlockDistruct<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InitDistruct({})", self.named_block.block.items)
    }
}

#[test]
fn parse_init_distruct() {
    check(" main {}..", |code| InitBlockDistruct {
        named_block: NamedBlock {
            name: Ident::new(1..=4, code),
            block: UnnamedBlock::new(vec![], [6, 7]),
        },
        distr: Distribution(Slice::new(8..=9, code)),
    });

    check_none::<InitBlockDistruct>(" main }..");
    check_none::<InitBlockDistruct>(" main {..");
    check_none::<InitBlockDistruct>(" main {}.");
    check_none::<InitBlockDistruct>(" main {}");
    check_none::<InitBlockDistruct>(" main .");
    check_none::<InitBlockDistruct>(" main ..");
}
