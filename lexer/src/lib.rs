//! Лексер парсит конструкции языка из строки кода с помощью трекера [`Code`].  
//! [`Code`] следит за исходной строкой кода, предотвращая ее клонировние следуя единному время жизни - временни жизни исходной строки:  
//!
//! - `'s` (source) - время жизни исходной строки  

#![doc(html_no_source)]
#![feature(type_alias_impl_trait)]
#![feature(associated_type_defaults)]
#![allow(unused)]
#![feature(extend_one)]
#![feature(macro_metavar_expr)]
#![feature(inherent_associated_types)]
#![feature(internal_output_capture)]

mod lexer2;
// mod lexer;
// use items::Items;
// pub use lexer::*;

// pub fn parse(source: &str) -> Items<'_> {
//     Items::parse(
//         &Code::new(source),
//         &mut Default::default(),
//         &mut Default::default(),
//     )
//     .unwrap()
// }
