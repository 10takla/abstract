#![feature(extend_one)]

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed, Meta,
    MetaList, Type, Variant,
};

#[proc_macro_derive(Parse, attributes(grammar))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);

    match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => {
            let Meta::List(MetaList { tokens, .. }) = &attrs.first().expect("expect attribute #[grammar()]").meta else {
                unreachable!()
            };

            let mut conformity = tokens.clone().into_iter().map(|v| {
                let TokenTree::Ident(ident) = v else {
                    unreachable!()
                };

                named
                    .iter()
                    .find(|Field { ty, .. }| {
                        let Type::Path(ty) = &ty else {unreachable!()};
                        let ty = &ty.path.segments.iter().next().unwrap().ident;
                        *ty == ident
                    }).unwrap()
            });

            let body = conformity
                .clone()
                .map(|Field { ty, ident, .. }| {
                    let Type::Path(ty) = &ty else {unreachable!()};
                        let ty = &ty.path.segments.iter().next().unwrap().ident;
                    quote! {
                        let #ident = #ty ::parse_and_consume(code)?;
                    }
                })
                .fold(proc_macro2::TokenStream::new(), |mut acc, stmt| {
                    if !acc.is_empty() {
                        acc.extend_one(quote! {
                            crate::lexer::items::shared::whitespaces::Whitespaces::parse_and_consume(code);
                        });
                    }
                    acc.extend_one(stmt);
                    acc
                });

            let fields = named.iter().map(|Field { ident, .. }| ident);

            let [first, last] = [
                &conformity.next().unwrap().ident,
                &conformity.last().unwrap().ident
            ];
            quote! {
                impl<'s> crate::lexer::Parse<'s> for #ident<'s> {
                    fn parse(code: &crate::lexer::Code<'s>) -> Option<Self> {
                        let code = &mut code.clone();

                        #body

                        Some(Self { #( #fields ),* })
                    }
                }

                impl crate::lexer::Slicable for #ident<'_> {
                    fn get_start(&self) -> usize {
                        self. #first .get_start()
                    }
                    fn get_end(&self) -> usize {
                        self. #last .get_end()
                    }
                }
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let (a, b): (Vec<_>, Vec<_>) = variants
                .into_iter()
                .map(|Variant { ident, fields, .. }| {
                    let attr = &fields.iter().next().unwrap();
                    (
                        quote! {
                            #attr => Self:: #ident,
                        },
                        quote! {
                            #ident
                        },
                    )
                })
                .unzip();

            quote! {
                impl<'s> crate::lexer::Parse<'s> for #ident<'s> {
                    fn parse(code: &crate::lexer::Code<'s>) -> Option<Self> {
                        crate::parse_variants!(
                            code
                            #( #a )*
                        )
                    }
                }

                impl crate::lexer::Slicable for #ident<'_> {
                    fn get_start(&self) -> usize {
                        match self {
                            #( Self:: #b (v) => v.get_start(), )*
                        }
                    }
                    fn get_end(&self) -> usize {
                        match self {
                            #( Self:: #b (v) => v.get_end(), )*
                        }
                    }
                }

                impl std::fmt::Display for #ident<'_> {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(
                            f,
                            "{}",
                            match self {
                                #( Self:: #b (v) => v.to_string(), )*
                            }
                        )
                    }
                }
            }
        }
        _ => unreachable!(),
    }
    .into()
}
