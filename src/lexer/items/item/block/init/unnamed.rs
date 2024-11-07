use super::{named::NamedBlock, Init};
use crate::lexer::{
    check, check_none,
    items::{
        item::{block::Block, ident::Ident, Item},
        shared::whitespaces::Whitespaces,
        Items,
    },
    Code, Parse, Slicable,
};
use std::fmt::Display;

#[derive(PartialEq, Debug, Hash, Eq, Clone)]
pub struct UnnamedBlock {
    pub items: Items,
    pub open_bracket_pos: usize,
    pub close_bracket_pos: usize,
}

impl UnnamedBlock {
    pub fn new(items: Vec<Item>, [open_bracket_pos, close_bracket_pos]: [usize; 2]) -> Self {
        Self {
            items: Items(items),
            open_bracket_pos,
            close_bracket_pos,
        }
    }
}

impl Parse for UnnamedBlock {
    fn parse(code: &Code) -> Option<Self> {
        let code = &mut code.clone();

        Whitespaces::parse_and_consume(code);
        let (i, char) = code.iter().next()?;
        let open_bracket_pos = (char == '{').then_some(i)?;
        code.consume(open_bracket_pos);

        let items = Items::parse(code).unwrap();
        if !items.is_empty() {
            code.end(&items);
        }

        Whitespaces::parse_and_consume(code);
        let (i, char) = code.iter().next()?;
        let close_bracket_pos = (char == '}').then_some(i)?;

        Some(Self {
            items,
            open_bracket_pos,
            close_bracket_pos,
        })
    }
}

impl Slicable for UnnamedBlock {
    fn get_end(&self) -> usize {
        self.close_bracket_pos
    }
}

impl Display for UnnamedBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block({})", self.items)
    }
}

#[test]
pub fn parse_block() {
    check(" {  } ", |_| UnnamedBlock::new(vec![], [1, 4]));
    check(" {}", |_| UnnamedBlock::new(vec![], [1, 2]));
    check("{}", |_| UnnamedBlock::new(vec![], [0, 1]));

    check("{asdasd asdasd asdasd}", |code| {
        UnnamedBlock::new(
            vec![
                Item::Ident(Ident::new([1, 6], code)),
                Item::Ident(Ident::new([8, 13], code)),
                Item::Ident(Ident::new([15, 20], code)),
            ],
            [0, 21],
        )
    });

    check("{asdasd asdasd asdasd { dsf } }", |code| {
        Block::Init(Init::Unnamed(UnnamedBlock::new(
            vec![
                Item::Ident(Ident::new([1, 6], code)),
                Item::Ident(Ident::new([8, 13], code)),
                Item::Block(Block::Init(Init::Named(NamedBlock {
                    name: Ident::new([15, 20], code),
                    block: UnnamedBlock {
                        items: Items(vec![Item::Ident(Ident::new([24, 26], code))]),
                        open_bracket_pos: 22,
                        close_bracket_pos: 28,
                    },
                }))),
            ],
            [0, 30],
        )))
    });

    // error
    check_none::<UnnamedBlock>("");
    check_none::<UnnamedBlock>(" ");
    check_none::<UnnamedBlock>("   ");
    check_none::<UnnamedBlock>(" } ");
    check_none::<UnnamedBlock>(" { ");
}
