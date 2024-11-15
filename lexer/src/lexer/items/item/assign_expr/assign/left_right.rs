use crate::lexer::{items::shared::whitespaces::Whitespaces, Code, Parse};
use macros::Slicable;
use std::marker::PhantomData;

#[derive(PartialEq, Debug, Clone, Hash, Eq, Slicable)]
pub struct LeftRight<'s, L: Parse<'s>, R: Parse<'s>> {
    #[start_]
    pub left: L,
    #[end]
    pub right: R,
    pub _marker: PhantomData<&'s ()>,
}

impl<'s, L: Parse<'s>, R: Parse<'s>> LeftRight<'s, L, R> {
    pub fn parse(
        code: &Code<'s>,
        mut middle_fn: impl FnMut(&mut Code<'s>) -> Option<usize>,
    ) -> Option<Self> {
        let code = &mut code.clone();

        let left = L::parse_and_consume(code)?;
        Whitespaces::parse_and_consume(code);
        let end = middle_fn(code)?;
        code.consume(end);
        Whitespaces::parse_and_consume(code);
        let right = R::parse(code)?;

        Some(Self {
            left,
            right,
            _marker: Default::default(),
        })
    }
}
