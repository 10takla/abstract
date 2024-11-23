#![feature(extend_one)]

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, ItemEnum, ItemStruct, Meta, MetaList, MetaNameValue, Type, Variant
};

#[proc_macro_derive(Parse, attributes(grammar, diag))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);

    let get_attr_type = |attrs: &Vec<Attribute>| {
        attrs
        .iter()
        .find_map(|attr| {
            if let Meta::List(MetaList { path, tokens, .. }) = &attr.meta {
                if path.is_ident("diag") {
                    if let Some(TokenTree::Ident(v)) = tokens.clone().into_iter().next() {
                        return Some(quote! {#v});
                    }
                }
            }
            None
        })
        .expect("Expect #[diag(...)]")
    };

    let diag_type = get_attr_type(&attrs);

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
                .map(|Field { ty, ident, attrs, .. }| {
                    let ty = {
                        let Type::Path(ty) = &ty else {unreachable!()};
                        &ty.path.segments.iter().next().unwrap().ident
                    };

                    let variant =  get_attr_type(attrs);

                    quote! {
                        let #ident = #ty ::diag_and_consume(code)
                        .map_err(|d| {
                            diags.extend(d.into_iter().map(|(i, d)| (i, Self::Diag:: #variant (d))));
                        })
                        .ok()?;
                    }
                })
                .fold(proc_macro2::TokenStream::new(), |mut acc, stmt| {
                    if !acc.is_empty() {
                        acc.extend_one(quote! {
                            crate::lexer::items::shared::whitespaces::Whitespaces::parse_and_consume(code, &mut vec![]);
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
                impl<'s> crate::lexer::DiagParse<'s> for #ident<'s> {
                    type Diag = #diag_type;

                    fn parse(code: &crate::lexer::Code<'s>, diags: &mut crate::lexer::Diags::<Self::Diag>) -> Option<Self> {
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
                    let field = &fields.iter().next().unwrap();
                    let Type::Path(p) = &field.ty else {unreachable!()};
                    let field = &p.path.segments.iter().next().unwrap().ident;
                    (
                        quote! {
                            #field ::diag(code).map(Self:: #ident),
                            diag: #ident
                        },
                        quote! {
                            #ident
                        },
                    )
                })
                .unzip();
            // panic!("{}", confirms.into_iter().map(|v| v.to_string()).collect::<String>());
            quote! {
                impl<'s> crate::lexer::DiagParse<'s> for #ident<'s> {
                    type Diag = #diag_type;

                    fn parse(code: &crate::lexer::Code<'s>, diags: &mut crate::lexer::Diags<Self::Diag>) -> Option<Self> {
                        crate::parse_variants!(
                            diag diags
                            #( #confirms );*
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

fn get_attr_type(attrs: &Vec<Attribute>, attr_str: &'static str) -> Result<TokenTree, String> {
    attrs
    .iter()
    .find_map(|attr| {
        if let Meta::List(MetaList { path, tokens, .. }) = &attr.meta {
            if path.is_ident(attr_str) {
                return tokens.clone().into_iter().next()
            }
        }
        None
    }).ok_or(format!("Expect attribute #[{attr_str}(...)]"))
}

#[proc_macro_derive(Diagn, attributes(name, diagn_expect))]
pub fn derive_diagn(input: TokenStream) -> TokenStream {
    let ItemEnum {
        ident, attrs, variants, ..
    } = parse_macro_input!(input);

    let get_literal_attr = |attrs| {
        get_attr_type(&attrs, "name").map(|a| {
            let TokenTree::Literal(l) = a else {
                unreachable!();
            };
            l
        })
    };

    let l = get_literal_attr(attrs).unwrap();
    let expects = variants.into_iter().map(|Variant {attrs, ident, ..} | {
        let l = get_attr_type(&attrs, "diagn_expect").unwrap();
        quote! {
            Self:: #ident => #l,
        }
    });

    quote!{
        impl crate::lexer::Diagn for #ident {
            const NAME: &'static str = #l;

            fn expect(&self, code: &Code, pos: usize) -> &'static str {
                match self {
                    #( #expects ) *
                }
            }
        }
    }.into()
}