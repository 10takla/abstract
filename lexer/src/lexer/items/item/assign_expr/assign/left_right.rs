use crate::lexer::{items::shared::whitespaces::Whitespaces, Code, Parse, Slicable};

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub struct LeftRight<L: Parse, R: Parse> {
    pub left: L,
    pub right: R,
}

impl<L: Parse, R: Parse> LeftRight<L, R> {
    pub fn parse(
        code: &Code,
        mut middle_fn: impl FnMut(&mut Code) -> Option<usize>,
    ) -> Option<Self> {
        let code = &mut code.clone();

        let left = L::parse_and_consume(code)?;
        Whitespaces::parse_and_consume(code);
        let end = middle_fn(code)?;
        code.consume(end);
        Whitespaces::parse_and_consume(code);
        let right = R::parse(code)?;

        Some(Self { left, right })
    }
}

impl<L: Parse, R: Parse> Slicable for LeftRight<L, R> {
    fn get_start(&self) -> usize {
        self.left.get_start()
    }
    fn get_end(&self) -> usize {
        self.right.get_end()
    }
}
