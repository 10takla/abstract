use crate::{
    lexer::{items::shared::whitespaces::Whitespaces, Code, DiagParse, Diags},
    recognizee, Parse, RecognizeParse, Recognized,
};
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

#[derive(PartialEq, Debug, Clone)]
pub enum LeftRightDiag<L, R> {
    Left(L),
    Right(R),
}

impl<
        's,
        LD: Clone + std::fmt::Debug,
        L: RecognizeParse<'s, Diag = LD> + Clone,
        RD: Clone + std::fmt::Debug,
        R: RecognizeParse<'s, Diag = RD> + Clone,
    > LeftRight<'s, L, R>
{
    // pub fn parse_m<MD, M: DiagParse<'s, Diag = MD>>(
    //     code: &Code<'s>,
    //     diags: &mut Diags<LeftRightDiag<LD, RD>>,
    // ) -> Option<Self> {
    //     let code = &mut code.clone();

    //     let left = L::diag_and_consume(code)
    //         .map_err(|d| {
    //             diags.extend(d.iter().cloned().map(|(i, d)| (i, LeftRightDiag::Left(d))));
    //         })
    //         .ok()?;

    //     Whitespaces::parse_and_consume(code, &mut vec![]);

    //     M::diag_and_consume(code).map_err(|d| {
    //         diags.extend(d.iter().cloned().map(|(i, d)| (i, LeftRightDiag::Left(d))));
    //     })
    //     .ok()?;

    //     Whitespaces::parse_and_consume(code, &mut vec![]);

    //     let right = R::diag(code)
    //         .map_err(|d| {
    //             diags.extend(d.iter().cloned().map(|(i, d)| (i, LeftRightDiag::Right(d))));
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
        recognized: &mut Recognized<'s>,
        mut middle_fn: impl FnMut(&mut Code<'s>) -> Option<usize>,
    ) -> Option<Self> {
        let code = &mut code.clone();

        let left = L::rec_and_consume(code, recognized)
            .map_err(|d| {
                diags.extend(d.iter().cloned().map(LeftRightDiag::Left));
            })
            .ok()?;

        Whitespaces::rec_and_consume(code, recognized);

        let end = middle_fn(code)?;
        code.consume(end);

        Whitespaces::rec_and_consume(code, recognized);

        let right = R::rec(code, recognized)
            .map_err(|d| {
                diags.extend(d.iter().cloned().map(LeftRightDiag::Right));
            })
            .ok()?;

        Some(Self {
            left,
            right,
            _marker: Default::default(),
        })
    }
}
