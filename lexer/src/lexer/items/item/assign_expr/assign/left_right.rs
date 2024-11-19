use crate::lexer::{items::shared::whitespaces::Whitespaces, Code, DiagParse, Diags};
use macros::Slicable;
use std::marker::PhantomData;

#[derive(PartialEq, Debug, Clone, Hash, Eq, Slicable)]
pub struct LeftRight<'s, L: DiagParse<'s>, R: DiagParse<'s>> {
    #[start_]
    pub left: L,
    #[end]
    pub right: R,
    pub _marker: PhantomData<&'s ()>,
}

#[derive(PartialEq, Debug)]
pub enum LeftRightDiag<L, R> {
    Left(L),
    Right(R),
}

impl<'s, LD, L: DiagParse<'s, Diag = LD>, RD, R: DiagParse<'s, Diag = RD>> LeftRight<'s, L, R> {
    // pub fn parse_m<MD, M: DiagParse<'s, Diag = MD>>(
    //     code: &Code<'s>,
    //     diags: &mut Diags<LeftRightDiag<LD, RD>>,
    // ) -> Option<Self> {
    //     let code = &mut code.clone();

    //     let left = L::diag_and_consume(code)
    //         .map_err(|d| {
    //             diags.extend(d.into_iter().map(|(i, d)| (i, LeftRightDiag::Left(d))));
    //         })
    //         .ok()?;

    //     Whitespaces::parse_and_consume(code, &mut vec![]);

    //     M::diag_and_consume(code).map_err(|d| {
    //         diags.extend(d.into_iter().map(|(i, d)| (i, LeftRightDiag::Left(d))));
    //     })
    //     .ok()?;

    //     Whitespaces::parse_and_consume(code, &mut vec![]);

    //     let right = R::diag(code)
    //         .map_err(|d| {
    //             diags.extend(d.into_iter().map(|(i, d)| (i, LeftRightDiag::Right(d))));
    //         })
    //         .ok()?;

    //     Some(Self {
    //         left,
    //         right,
    //         _marker: Default::default(),
    //     })
    // }

    pub fn parse(
        code: &Code<'s>,
        diags: &mut Diags<LeftRightDiag<LD, RD>>,
        mut middle_fn: impl FnMut(&mut Code<'s>) -> Option<usize>,
    ) -> Option<Self> {
        let code = &mut code.clone();

        let left = L::diag_and_consume(code)
            .map_err(|d| {
                diags.extend(d.into_iter().map(|(i, d)| (i, LeftRightDiag::Left(d))));
            })
            .ok()?;

        Whitespaces::parse_and_consume(code, &mut vec![]);

        let end = middle_fn(code)?;
        code.consume(end);

        Whitespaces::parse_and_consume(code, &mut vec![]);

        let right = R::diag(code)
            .map_err(|d| {
                diags.extend(d.into_iter().map(|(i, d)| (i, LeftRightDiag::Right(d))));
            })
            .ok()?;

        Some(Self {
            left,
            right,
            _marker: Default::default(),
        })
    }
}
