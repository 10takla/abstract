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
#![feature(generic_const_exprs)]
#![feature(min_const_generics)]

use parser::{
    
    CommonError, CommonRecog,
};
pub use utils::{args::*, print::*};
use language::{Enum1, Items};

pub mod language;
pub mod parser;
mod utils;

pub fn parse(source: &str) -> (Vec<Enum1>, Vec<CommonError>) {
    let mut t = source.into();
    (Items::recog(&t).unwrap(), t.errors.clone().borrow().clone())
}
