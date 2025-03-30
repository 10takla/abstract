//! Лексер парсит конструкции языка из строки кода с помощью трекера [`Code`].  
//! [`Code`] следит за исходной строкой кода, предотвращая ее клонировние следуя единному время жизни - временни жизни исходной строки:  
//!
//! - `'s` (source) - время жизни исходной строки  

#![doc(html_no_source)]
#![allow(unused)]
#![feature(
    let_chains,
    if_let_guard,
    type_alias_impl_trait,
    associated_type_defaults,
    extend_one,
    macro_metavar_expr,
    internal_output_capture,
    negative_impls,
    min_specialization,
    marker_trait_attr,
    adt_const_params,
    array_try_map,
    trait_upcasting
)]

use language::*;
use parser::{CommonError, CommonRecog, Slice};
pub use utils::{args::*, print::*};

pub mod language;
pub mod parser;
mod utils;

pub fn parse(source: &str) -> (Vec<Item>, Vec<(Slice, std::string::String)>) {
    let ctxt = source.into();
    let v = Items::recog(&ctxt).unwrap();
    (
        v.data.into_iter().fold(Default::default(), |mut acc, v| {
            match v {
                Enum0::V0(v) => {
                    acc.push(v);
                }
                Enum0::V1(v) => {}
            }
            acc
        }),
        {
            let v = ctxt.errors.borrow().clone();
            v
        },
    )
}
