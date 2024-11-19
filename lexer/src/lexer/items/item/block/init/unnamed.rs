use super::{named::NamedBlock, Init};
use crate::{
    items::item::{
        block::Block,
        ident::{Ident, IdentDiag},
        ItemDiag,
    },
    lexer::{
        check, check_diag, check_none,
        items::{item::Item, shared::whitespaces::Whitespaces, Items},
        Code, DiagParse, Diags, Slicable,
    },
};
use std::{fmt::Display, ops::RangeInclusive};

#[derive(PartialEq, Debug, Hash, Eq, Clone)]
pub struct UnnamedBlock<'s> {
    pub items: Items<'s>,
    pub open_bracket_pos: usize,
    pub close_bracket_pos: usize,
}

#[derive(PartialEq, Debug)]
pub enum UnnamedBlockDiag {
    StartsOpenBracket,
    EndsOpenBracket,
    Items(Box<ItemDiag>),
}

impl<'s> DiagParse<'s> for UnnamedBlock<'s> {
    type Diag = UnnamedBlockDiag;

    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>) -> Option<Self> {
        let code = &mut code.clone();

        Whitespaces::parse_and_consume(code, &mut vec![]);
        let (i, char) = code.iter().next()?;
        let open_bracket_pos = (char == '{').then_some(i).or_else(|| {
            diags.push((i, UnnamedBlockDiag::StartsOpenBracket));
            None
        })?;
        code.consume(open_bracket_pos);

        let mut d = vec![];
        let items = Items::parse(code, &mut d).unwrap();
        if !items.is_empty() {
            code.end(&items);
        }

        diags.extend(
            d.into_iter()
                .map(|(i, dd)| (i, UnnamedBlockDiag::Items(dd.into()))),
        );

        Whitespaces::parse_and_consume(code, &mut vec![]);
        let (i, char) = code.iter().next().or_else(|| {
            diags.push((code.len() - 1, UnnamedBlockDiag::EndsOpenBracket));
            None
        })?;
        let close_bracket_pos = (char == '}').then_some(i).or_else(|| {
            diags.push((i, UnnamedBlockDiag::EndsOpenBracket));
            None
        })?;

        Some(Self {
            items,
            open_bracket_pos,
            close_bracket_pos,
        })
    }
}

impl Slicable for UnnamedBlock<'_> {
    fn get_slice(&self) -> std::ops::RangeInclusive<usize> {
        RangeInclusive::new(self.open_bracket_pos, self.close_bracket_pos)
    }
}

impl Display for UnnamedBlock<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block({})", self.items)
    }
}

impl<'s> UnnamedBlock<'s> {
    pub fn new(items: Vec<Item<'s>>, [open_bracket_pos, close_bracket_pos]: [usize; 2]) -> Self {
        Self {
            items: Items(items),
            open_bracket_pos,
            close_bracket_pos,
        }
    }
}

#[test]
pub fn parse() {
    check(" {  } ", |_| UnnamedBlock::new(vec![], [1, 4]));
    check(" {}", |_| UnnamedBlock::new(vec![], [1, 2]));
    check("{}", |_| UnnamedBlock::new(vec![], [0, 1]));

    check("{asdasd asdasd asdasd}", |code| {
        UnnamedBlock::new(
            vec![
                Item::Ident(Ident::new(1..=6, code)),
                Item::Ident(Ident::new(8..=13, code)),
                Item::Ident(Ident::new(15..=20, code)),
            ],
            [0, 21],
        )
    });

    check("{asdasd asdasd asdasd { dsf } }", |code| {
        Block::Init(Init::Unnamed(UnnamedBlock::new(
            vec![
                Item::Ident(Ident::new(1..=6, code)),
                Item::Ident(Ident::new(8..=13, code)),
                Item::Block(Block::Init(Init::Named(NamedBlock {
                    name: Ident::new(15..=20, code),
                    block: UnnamedBlock {
                        items: Items(vec![Item::Ident(Ident::new(24..=26, code))]),
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

#[test]
fn diag() {
    check_diag::<UnnamedBlockDiag, UnnamedBlock>(
        "s",
        vec![(0, UnnamedBlockDiag::StartsOpenBracket)],
    );
    check_diag::<UnnamedBlockDiag, UnnamedBlock>(
        "{dd",
        vec![(2, UnnamedBlockDiag::EndsOpenBracket)],
    );
    check_diag::<UnnamedBlockDiag, UnnamedBlock>(
        "  {  ",
        vec![(4, UnnamedBlockDiag::EndsOpenBracket)],
    );
    check_diag::<UnnamedBlockDiag, UnnamedBlock>(
        "  { 23d ",
        vec![
            (
                6,
                UnnamedBlockDiag::Items(ItemDiag::Ident(IdentDiag::StartsWithNotNumber).into()),
            ),
            (7, UnnamedBlockDiag::EndsOpenBracket),
        ],
    );
}
