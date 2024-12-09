#![feature(extend_one)]

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{
    parse, parse_macro_input, punctuated::Punctuated, Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, ItemEnum, Meta, MetaList, Token, Type, Variant
};

#[proc_macro_derive(Parse, attributes(grammar, diag))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse(input.clone()).unwrap();

    let diag_type = get_attr_type(&attrs, "diag").unwrap();

    let (
        body_parse,
        other,
    ) = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => {
            let conformity = get_grammar_data(&attrs, &named).unwrap();
            
            let body = conformity
                .map(|Field { ty, ident, attrs, .. }| {
                    let ty = {
                        let Type::Path(ty) = &ty else {unreachable!()};
                        &ty.path.segments.iter().next().unwrap().ident
                    };

                    let variant =  get_attr_type(attrs, "diag").unwrap();

                    quote! {
                        let #ident = #ty ::rec_and_consume(code, recognized)
                        .map_err(|d| {
                            diags.extend(d.iter().cloned().map(Self::Diag:: #variant));
                        })
                        .ok()?;
                    }
                })
                .fold(proc_macro2::TokenStream::new(), |mut acc, stmt| {
                    if !acc.is_empty() {
                        acc.extend_one(quote! {
                            crate::lexer::items::shared::whitespaces::Whitespaces::rec_and_consume(code, recognized);
                        });
                    }
                    acc.extend_one(stmt);
                    acc
                });

            let fields = named.iter().map(|Field { ident, .. }| ident);

            (
                quote! {
                    let code = &mut code.clone();
                    let recognized = &mut recognized.clone();

                    #body

                    Some(Self { #( #fields ),* })
                },
                quote! {
                    // impl<'s> crate::lexer::RecognizeParse<'s> for #ident<'s> {}
                },
            )
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let (confirms, variants): (Vec<_>, Vec<_>) = variants
                .into_iter()
                .map(|Variant { ident, fields, .. }| {
                    let field = &fields.iter().next().unwrap();
                    let Type::Path(p) = &field.ty else {
                        unreachable!()
                    };
                    let field = &p.path.segments.iter().next().unwrap().ident;
                    (
                        quote! {
                            #field ::rec(code, recognized).map(Self:: #ident),
                            diag: #ident
                        },
                        ident,
                    )
                })
                .unzip();
            (
                quote! {
                    crate::parse_variants!(
                        diag diags
                        #( #confirms );*
                    )
                },
                quote! {
                    impl<'s> crate::lexer::SelectionParse<'s> for #ident<'s> {}
                    
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
                },
            )
        }
        _ => unreachable!(),
    };

    let impl_slicable: proc_macro2::TokenStream = derive_slicable(input.clone()).into();

    quote!{
        impl<'s> crate::lexer::Parse<'s> for #ident<'s> {
            type Diag = #diag_type;

            fn parse(code: &crate::lexer::Code<'s>, diags: &mut crate::lexer::Diags<Self::Diag>, recognized: &mut crate::lexer::Recognized<'s>) -> Option<Self> {
                use crate::lexer::DiagParse;
                use crate::lexer::{RecognizeParse, SelectionParse};
                #body_parse
            }
        }
        impl<'s> crate::lexer::DiagParse<'s> for #ident<'s> {}
        #other

        #impl_slicable
    }
    .into()
}

#[proc_macro_derive(Slicable, attributes(grammar, slice, start_, end))]
pub fn derive_slicable(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        attrs,
        generics,
        ..
    } = parse(input).unwrap();

    let body = match data {
        Data::Enum(DataEnum { variants, .. }) => {
            let variants = variants.iter().map(|Variant { ident, .. }| ident);
            quote! {
                match self {
                    #( Self:: #variants (v) => v.get_slice(), )*
                }
            }
        }
        Data::Struct(DataStruct { fields, .. }) => {
            let body_from_grammar = if let Fields::Named(FieldsNamed { named, .. }) = &fields {
                get_grammar_data(&attrs, &named).map(|mut conformity| {
                    let [first, last] = [
                        &conformity.next().unwrap().ident,
                        &conformity.last().unwrap().ident,
                    ];
                    quote! {
                        std::ops::RangeInclusive::new(
                            self. #first .get_start(),
                            self. #last .get_end(),
                        )
                    }
                }).ok()
            } else {
                None
            };
            body_from_grammar.unwrap_or_else(|| {
                match fields {
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
                }
            })            
        },
        _ => unreachable!(),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics crate::lexer::Slicable for #ident #ty_generics  #where_clause {
            fn get_slice(&self) -> std::ops::RangeInclusive<usize> {
                #body
            }
        }
    }
    .into()
}

fn get_grammar_data<'a>(attrs: &Vec<Attribute>, named: &'a Punctuated<Field, Token![,]>) -> Result<impl Iterator::<Item = &'a Field>, String> {

    get_attr_list(attrs, "grammar").map(|tokens| {
        tokens.clone().into_iter().map(|v| {
            let TokenTree::Ident(ident) = v else {
                unreachable!()
            };
    
            named
                .iter()
                .find(|Field { ty, .. }| {
                    let Type::Path(ty) = &ty else { unreachable!() };
                    let ty = &ty.path.segments.iter().next().unwrap().ident;
                    *ty == ident
                })
                .unwrap()
        })
    })
    
}

#[proc_macro_derive(Diagn, attributes(name, diagn_expect))]
pub fn derive_diagn(input: TokenStream) -> TokenStream {
    let ItemEnum {
        ident,
        attrs,
        variants,
        ..
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
    let expects = variants.into_iter().map(|Variant { attrs, ident, .. }| {
        let l = get_attr_type(&attrs, "diagn_expect").unwrap();
        quote! {
            Self:: #ident => #l,
        }
    });

    quote! {
        impl crate::lexer::Diagn for #ident {
            const NAME: &'static str = #l;

            fn expect(&self, code: &Code, pos: usize) -> &'static str {
                match self {
                    #( #expects ) *
                }
            }
        }
    }
    .into()
}

fn get_attr_type(attrs: &Vec<Attribute>, attr_str: &'static str) -> Result<TokenTree, String> {
    get_attr_list(attrs, attr_str).and_then(|tokens| {
        tokens
            .clone()
            .into_iter()
            .next()
            .ok_or(format!("Expect one item in #[{attr_str}(...)]"))
    })
}

fn get_attr_list<'a>(
    attrs: &'a Vec<Attribute>,
    attr_str: &'static str,
) -> Result<&'a proc_macro2::TokenStream, String> {
    attrs
        .iter()
        .find_map(|attr| {
            if let Meta::List(MetaList { path, tokens, .. }) = &attr.meta {
                if path.is_ident(attr_str) {
                    return Some(tokens);
                }
            }
            None
        })
        .ok_or(format!("Expect attribute #[{attr_str}(...)]"))
}

#[proc_macro_derive(RecognizeParse)]
pub fn derive_rec(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        ..
    } = parse_macro_input!(input);
    quote! {
        impl<'s> crate::lexer::RecognizeParse<'s> for #ident<'s> {
            fn rec(code: &crate::lexer::Code<'s>, recognized: &mut crate::lexer::Recognized<'s>) -> Result<Self, Diags<Self::Diag>> {
                use crate::lexer::{CacheKey, CacheItems};
                recognized
                    .get(&CacheKey:: #ident)
                    .and_then(|v| {
                        if let CacheItems:: #ident (v) = v {
                            Some(v.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        let res = Self::diag(code, recognized);
                        recognized.insert(CacheKey:: #ident, CacheItems:: #ident (res.clone()));
                        res
                    })
            }
        }
    }.into()
}

#[proc_macro_attribute]
pub fn test_with_sub(input: TokenStream, annoted: TokenStream) -> TokenStream {
    quote! {

    }.into()
}