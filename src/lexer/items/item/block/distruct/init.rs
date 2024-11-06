use crate::lexer::{
    check, check_none,
    items::{
        item::{
            block::init::{named::NamedBlock, unnamed::UnnamedBlock},
            ident::Ident,
        },
        shared::distribution::Distribution,
    },
    Code, Parse, Slicable, Slice,
};
use macros::Parse;
use std::fmt::Display;

#[derive(PartialEq, Debug, Parse)]
#[grammar(
    NamedBlock Distribution
)]
pub struct InitBlockDistruct {
    named_block: NamedBlock,
    distr: Distribution,
}

impl Display for InitBlockDistruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InitDistruct({})", self.named_block.block.items)
    }
}

#[test]
fn parse_init_distruct() {
    check(" main {}..", |code| InitBlockDistruct {
        named_block: NamedBlock {
            name: Ident::new([1, 4], code),
            block: UnnamedBlock::new(vec![], [6, 7]),
        },
        distr: Distribution(Slice::new([8, 9], code)),
    });

    check_none::<InitBlockDistruct>(" main }..");
    check_none::<InitBlockDistruct>(" main {..");
    check_none::<InitBlockDistruct>(" main {}.");
    check_none::<InitBlockDistruct>(" main {}");
    check_none::<InitBlockDistruct>(" main .");
    check_none::<InitBlockDistruct>(" main ..");
}
