#![feature(let_chains)]
#![allow(unused)]

use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, Lit, Path
};

struct T(Ident, Option<Path>, Lit);
impl Parse for T {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(
            Parse::parse(input)?,
            Parse::parse(input).ok(),
            Parse::parse(input)?,
        ))
    }
}

#[proc_macro]
pub fn distruct(input: TokenStream) -> TokenStream {
    let T(type_, k, v) = parse_macro_input!(input);
    let Lit::Int(v) = v else { unreachable!() };
    let v = v.base10_parse::<usize>().unwrap();
    match type_.to_string().as_str() {
        "enum_" => {
            let name = k.unwrap();
            let v = (0..v).map(|i| Ident::new(&format!("V{i}"), Span::call_site()));
            quote! {
                impl Distruct for #name {
                    fn distruct(&self, vec: &mut DistrIter) {
                        use #name::*;
                        match self {
                            #(
                               #v (v) => v.distruct(vec)
                            ),*
                        }
                    }
                }
            }
        }
        "struct_" => {
            let v = (2..=v).map(|v| {
                let a = (0..v)
                    .clone()
                    .map(Literal::usize_unsuffixed)
                    .collect::<Vec<_>>();
                quote! {
                    paste::paste! {
                        impl<#([<T #a>]: Distruct),*> Distruct for  (#([<T #a>]),*) {
                            fn distruct(&self, vec: &mut DistrIter) {
                                #(
                                    self.#a.distruct(vec);
                                )*
                            }
                        }
                    }
                }
            });

            quote! {#(#v)*}
        }
        _ => {
            unreachable!();
        }
    }
    .into()
}
