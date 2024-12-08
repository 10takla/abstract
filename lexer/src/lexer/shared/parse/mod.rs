pub mod recognizee;
pub mod diag;

use super::{code::Code, slice::Slicable};
use core::error;
use diag::Diags;
use recognizee::Recognized;
use std::{array::IntoIter, fmt::Debug};
use std_reset::prelude::Deref;

#[macro_export]
macro_rules! parse_variants {
    (diag $diags:ident  $expr1:expr, diag: $diag1:ident; $( $expr:expr, diag: $diag:ident ); + $(;)?) => {
        crate::parse_variants!(
            $diags  $expr1, diag: $diag1
        )
            $(
                .or_else(|_| {
                    crate::parse_variants!(
                        $diags  $expr, diag: $diag
                    )
                })
            )+
            .ok()
    };
    ($diags:ident  $expr:expr, diag: $diag:ident) => {
        $expr
            .map_err(|d| {
                $diags.extend(d.iter().cloned().map(Self::Diag:: $diag));
            })
    }
}

pub trait Parse<'s>: Slicable + Sized + Debug {
    type Diag;
    type Diags: Default = Diags<Self::Diag>;
    fn parse(
        code: &Code<'s>,
        diags: &mut Self::Diags,
        recognized: &mut Recognized<'s>,
    ) -> Option<Self>;

    fn parse_and_consume(
        code: &mut Code<'s>,
        diags: &mut Self::Diags,
        recognized: &mut Recognized<'s>,
    ) -> Option<Self> {
        Self::parse(code, diags, recognized).map(|v| v.consume(code, recognized))
    }

    fn consume(self, code: &mut Code<'s>, recognized: &mut Recognized<'s>) -> Self {
        code.end(&self);
        // *recognized = Default::default();
        self
    }
}
