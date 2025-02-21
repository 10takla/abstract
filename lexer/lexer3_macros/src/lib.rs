#![allow(unused)]

use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use syn::{parse_macro_input, Fields, FieldsUnnamed, Item, ItemEnum, ItemStruct, Variant};

#[proc_macro_derive(Spanable)]
pub fn tmp(input: TokenStream) -> TokenStream {
    let (ident, body) = match parse_macro_input!(input) {
        Item::Enum(ItemEnum {
            variants, ident, ..
        }) => {
            let variants = variants.into_iter().map(|v| v.ident).collect::<Vec<_>>();
            (
                ident,
                quote! {
                    match self {
                        #( Self::#variants(v) => v.span() ),*
                    }
                },
            )
        }
        Item::Struct(ItemStruct { fields, ident, .. }) => {
            let Fields::Unnamed(FieldsUnnamed { unnamed, .. }) = fields else {
                unreachable!()
            };

            let [start, end] = [0, unnamed.len() - 1].map(Literal::usize_unsuffixed);
            (
                ident,
                quote! {
                    self.#start.span().start..self.#end.span().end
                },
            )
        }
        _ => unreachable!(),
    };

    quote! {
        impl Spanable for #ident {
            fn span(&self) -> Slice {
                #body
            }
        }
    }
    .into()
}
