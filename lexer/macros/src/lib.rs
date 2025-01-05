#![feature(extend_one)]
#![feature(let_chains)]
#![feature(proc_macro_diagnostic)]

mod constructor;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

#[proc_macro]
pub fn constructor(input: TokenStream) -> TokenStream {
    constructor::constructor(input)
}

#[proc_macro_attribute]
pub fn parse_test(_: TokenStream, annoted: TokenStream) -> TokenStream {
    let fn_: ItemFn = parse_macro_input!(annoted);
    let ident = &fn_.sig.ident;
    let t_ident = Ident::new(&format!("{}_", ident), ident.span());
    quote! {
        #[test]
        fn #ident() {
            #t_ident::#ident(cli_args());
        }
        mod #t_ident {
            use super::*;
            pub #fn_
        }
    }
    .into()
}
