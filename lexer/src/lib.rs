//! Лексер парсит конструкции языка из строки кода с помощью трекера [`Code`].  
//! [`Code`] следит за исходной строкой кода, предотвращая ее клонировние следуя единному время жизни - временни жизни исходной строки:  
//!
//! - `'s` (source) - время жизни исходной строки  

#![feature(let_chains)]
#![feature(if_let_guard)]
#![doc(html_no_source)]
#![feature(type_alias_impl_trait)]
#![feature(associated_type_defaults)]
#![allow(unused)]
#![feature(extend_one)]
#![feature(macro_metavar_expr)]
#![feature(inherent_associated_types)]
#![feature(internal_output_capture)]
#![feature(negative_impls)]
#![feature(specialization)]
#![feature(min_specialization)]
#![feature(marker_trait_attr)]
#![feature(adt_const_params)]
#![feature(array_try_map)]
#![feature(trait_upcasting)]
#![feature(negative_bounds)]

pub mod lexer2;
pub mod lexer3;

use lexer2::{cache_and_diags::diag::Diag, Items, SequenceRecog, ParseArgs};

pub fn parse(source: &str) -> (Items, Vec<Diag>) {
    let mut t: ParseArgs = source.into();
    (
        Items::recog(&mut t, 0),
        t.c_a_d.clone().borrow().errors.clone(),
    )
}

// mod lexer;
// pub use lexer::*;

// pub fn parse(source: &str) -> Items<'_> {
//     Items::parse(
//         &Code::new(source),
//         &mut Default::default(),
//         &mut Default::default(),
//     )
//     .unwrap()
// }
