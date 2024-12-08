use super::{Diags};
use crate::{
    items::{
        item::{
            assign_expr::{
                assign::{Assign, AssignDiag},
                assign_and::{AssignAnd, AssignAndDiag},
                literal::{
                    number::{Number, NumberDiag},
                    string::{String, StringDiag},
                    Literal, LiteralDiag,
                },
                AssignExpr, AssignExprDiag,
            },
            block::{
                distruct::{
                    call::{CallBlockDistruct, CallBlockDistructDiag},
                    init::{InitBlockDistruct, InitBlockDistructDiag},
                    BlockDistruct, BlockDistructDiag,
                },
                init::{
                    named::{NamedBlock, NamedBlockDiag},
                    unnamed::{UnnamedBlock, UnnamedBlockDiag},
                    InitBlock, InitBlockDiag,
                },
                Block, BlockDiag,
            },
            ident::{Ident, IdentDiag},
            Item, ItemDiag,
        },
        shared::{
            distribution::{Distribution, DistributionDiag},
            whitespaces::{Whitespaces, WhitespacesDiag},
        },
    },
    lexer::shared::code::Code, DiagParse,
};
use paste::paste;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
};
use std_reset::prelude::Deref;

pub trait SelectionParse<'s>: DiagParse<'s> {
    fn rec_(code: &Code<'s>, recognized: &mut Recognized<'s>) -> Result<Self, Self::Diags> {
        Self::diag(code, recognized).map(|v| v)
    }
    fn rec_and_consume_(
        code: &mut Code<'s>,
        recognized: &mut Recognized<'s>,
    ) -> Result<Self, Self::Diags> {
        Self::diag_and_consume(code, recognized).map(|v| {
            *recognized = Default::default();
            v.consume(code, recognized)
        })
    }
}

pub trait RecognizeParse<'s>: DiagParse<'s>
where
    Self::Diag: Clone,
    Self: Clone,
{
    fn rec(code: &Code<'s>, recognized: &mut Recognized<'s>) -> Result<Self, Diags<Self::Diag>>;
    fn rec_and_consume(
        code: &mut Code<'s>,
        recognized: &mut Recognized<'s>,
    ) -> Result<Self, Diags<Self::Diag>> {
        Self::rec(code, recognized).map(|v| v.consume(code, recognized))
    }
}

type R<A, B> = Result<A, Diags<B>>;
macro_rules! fast {
    ($($name:ident), + $(,)?) => {
        #[derive(Clone, Debug)]
        pub enum CacheItems<'s> {
            $(
                $name(R<$name<'s>, paste!{ [<$name Diag>] } >),
            )+
        }

        #[derive(Hash, PartialEq, Eq, Debug, Clone)]
        pub enum CacheKey {
            $(
                $name
            ),+
        }
        $(
            impl<'s> crate::lexer::RecognizeParse<'s> for $name<'s> {
                fn rec(code: &crate::lexer::Code<'s>, recognized: &mut crate::lexer::Recognized<'s>) -> Result<Self, Diags<Self::Diag>> {
                    use crate::lexer::{CacheKey, CacheItems};
                    // dbg!((&recognized, CacheKey::$name));
                    // recognized.get()
                    //     .and_then(|v| {
                    //         if let CacheItems::$name(v) = v {
                    //             dbg!(v);
                    //             Some(v.clone())
                    //         } else {
                    //             None
                    //         }
                    //     }).map(|v| {
                    //         recognized.ptr += 1;
                    //         v
                    //     })
                    //     .unwrap_or_else(|| {
                    //         recognized.ptr = 0;
                            let res = Self::diag(code, recognized);
                            // recognized.push(dbg!(CacheItems::$name(res.clone())));
                            res
                        // })
                }
            }
        )+

    };
}
fast!(
    Literal,
    Number,
    String,
    Ident,
    AssignExpr,
    AssignAnd,
    Assign,
    Block,
    InitBlock,
    NamedBlock,
    UnnamedBlock,
    BlockDistruct,
    InitBlockDistruct,
    CallBlockDistruct,
    Item,
    Distribution,
    Whitespaces
);

#[derive(Default, Deref, Clone, Debug)]
pub struct Recognized<'s> {
    #[deref]
    cache: Vec<CacheItems<'s>>,
    ptr: usize,
}

impl<'s> Recognized<'s> {
    fn get(&self) -> Option<&CacheItems<'s>> {
        self.cache.get(self.ptr)
    }
}
