#![feature(let_chains)]
#![allow(unused)]

use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::{collections::HashSet, default};
use syn::{
    parse::{Parse, ParseBuffer, ParseStream},
    parse_macro_input,
    punctuated::{IntoIter, Iter, Punctuated},
    token::{Brace, Bracket, Break, Paren},
    AngleBracketedGenericArguments, Attribute, Expr, Fields, FieldsUnnamed, Ident, ItemEnum,
    ItemStruct, Lit, Meta, MetaList, PathArguments, Token, Variant,
};

#[proc_macro_derive(Spanable)]
pub fn spanable_derive(input: TokenStream) -> TokenStream {
    let (ident, body) = match parse_macro_input!(input) {
        syn::Item::Enum(ItemEnum {
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
        syn::Item::Struct(ItemStruct { fields, ident, .. }) => {
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

mod constructor;
#[proc_macro]
pub fn constructor(input: TokenStream) -> TokenStream {
    constructor::constructor(input)
}

mod peg_grammar;
#[proc_macro]
pub fn peg_grammar(input: TokenStream) -> TokenStream {
    peg_grammar::peg_grammar(input)
}

#[proc_macro_derive(EnumRecog, attributes(ty))]
pub fn enum_recog(input: TokenStream) -> TokenStream {
    let ItemEnum {
        variants, ident, ..
    } = parse_macro_input!(input);

    let count = variants.len();

    let b = variants.iter().map(|Variant { attrs, fields, .. }| {
        let field = {
            let Fields::Unnamed(v) = &fields else {
                unreachable!()
            };
            if v.unnamed.len() != 1 {
                unreachable!()
            }
            let v = v.unnamed[0].clone();
            
            attrs.iter()
                .find_map(|attr| {
                    if attr.meta.path().is_ident("ty") {
                        if let Meta::List(meta) = &attr.meta {
                            return Some(meta.tokens.clone());
                        }
                    }
                    None
                })
                .unwrap_or(v.ty.to_token_stream())
                
        };

        field
    });
    let a = variants.iter().map(|v| v.ident.clone());

    quote! {
        impl EnumRecog for #ident {
            type Output = Self;
            fn structure_assembling<'a>(
                ctxt: &'a Ctxt,
            ) -> Vec<Box<dyn core::ops::Fn() -> Result<Self::Output, CommonError> + 'a>> {
                vec![
                    #(
                        Box::new(|| <#b>::recog(ctxt).map(Self::#a))
                    ),*
                ]
            }
        }
    }
    .into()
}

fn find_attr(attrs: Vec<Attribute>, pref: &str) -> Option<Expr> {
    attrs.iter().find_map(|attr| {
        if attr.meta.path().is_ident(pref) {
            if let Meta::NameValue(meta) = &attr.meta {
                return Some(meta.value.clone());
            }
        }
        None
    })
}

#[proc_macro_derive(RegularToken, attributes(reg_expr))]
pub fn regular_token(input: TokenStream) -> TokenStream {
    let ItemStruct { ident, attrs, .. } = parse_macro_input!(input);

    let reg_expr = find_attr(attrs, "reg_expr")
        .and_then(|v| {
            if let Expr::Lit(v) = v {
                if let Lit::Str(v) = v.lit {
                    return Some(v);
                }
            }
            None
        })
        .unwrap();

    quote! {
        impl RegularToken for #ident {
            const REG_EXPR: &'static str = #reg_expr;
        }
    }
    .into()
}
