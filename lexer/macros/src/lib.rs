#![feature(extend_one)]

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
    FieldsUnnamed, Ident, ItemStruct, Meta, MetaList, Type, Variant,
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
                    fn get_slice(&self) -> std::ops::RangeInclusive<usize> {
                        std::ops::RangeInclusive::new(
                            self. #first .get_start(),
                            self. #last .get_end(),
                        )
                    }
                }
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let (confirms, variants): (Vec<_>, Vec<_>) = variants
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
                            #( #confirms )*
                        )
                    }
                }

                impl crate::lexer::Slicable for #ident<'_> {
                    fn get_slice(&self) -> std::ops::RangeInclusive<usize> {
                        match self {
                            #( Self:: #variants (v) => v.get_slice(), )*
                        }
                    }
                }

                impl std::fmt::Display for #ident<'_> {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(
                            f,
                            "{}",
                            match self {
                                #( Self:: #variants (v) => v.to_string(), )*
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

#[proc_macro_derive(Slicable, attributes(slice, start_, end))]
pub fn derive_slicable(input: TokenStream) -> TokenStream {
    let ItemStruct {
        ident,
        generics,
        fields,
        ..
    } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = match fields {
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
            quote! {self.0.get_slice()}
        }
        Fields::Named(FieldsNamed { named, .. }) => {
            let acc = named.iter().fold(
                (None, None, None),
                |(mut slice, mut start, mut end), Field { attrs, ident, .. }| {
                    let fast = |at| attrs.iter().any(|attr| attr.meta.path().is_ident(at));
                    let check = |a: Option<&Ident>, b| {
                        if a.is_some() {
                            panic!("#[{b}] already defined earlier");
                        }
                    };

                    if fast("slice") {
                        check(start, "start_");
                        check(end, "end");
                        slice = Some(ident.as_ref().unwrap())
                    }
                    if fast("start_") {
                        check(slice, "slice");
                        start = Some(ident.as_ref().unwrap())
                    }
                    if fast("end") {
                        check(slice, "slice");
                        end = Some(ident.as_ref().unwrap())
                    }
                    (slice, start, end)
                },
            );

            match acc {
                (Some(ident), None, None) => quote! {self. #ident .get_slice()},
                (None, Some(start), Some(end)) => {
                    quote! {std::ops::RangeInclusive::new(self. #start .get_start(), self. #end .get_end())}
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    };

    quote! {
        impl #impl_generics crate::lexer::Slicable for #ident #ty_generics  #where_clause {
            fn get_slice(&self) -> std::ops::RangeInclusive<usize> {
                #body
            }
        }
    }
    .into()
}
