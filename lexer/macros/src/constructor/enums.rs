use super::{check_pass_fail, fast_ident, fast_ident2, fast_puncts, tmp, tmp3, tmp5, COMMON};
use proc_macro2::{token_stream::IntoIter, Group, Ident, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::{
    iter::Peekable,
    panic::{catch_unwind, AssertUnwindSafe},
};
use syn::{custom_punctuation, parse2, LitStr};

pub fn enums(mut iter: IntoIter) -> Vec<(Ident, Vec<Ident>)> {
    let mut vec = vec![];
    while iter.clone().next().is_some() {
        vec.push(tmp3(&mut iter, enum_recognize).unwrap());
    }
    vec
}

pub fn enum_recognize(iter: &mut Peekable<IntoIter>) -> ((Ident, Vec<Ident>), usize) {
    let mut counter = 0;

    let name = fast_ident2(iter).unwrap();
    counter += 1;

    fast_puncts("->", iter).unwrap();
    counter += 2;

    let mut items = vec![];
    while let Some(v) = iter.next() {
        let v = fast_ident(&v).unwrap();
        counter += 1;
        items.push(v);
        let Ok(_) = fast_puncts("|", iter) else {
            break;
        };
        counter += 1;
    }
    if items.len() < 2 {
        panic!("Expect minimum 2 items")
    }
    ((name, items), counter)
}

pub fn enum_tokens(name: &Ident, items: &Vec<Ident>, items2: &Vec<Ident>) -> TokenStream2 {
    let common = &*COMMON;

    let tmp = {
        let (first, other) = {
            let item_recog = |item| {
                if items2.contains(item) {
                    quote! {
                        let v = #item::recog(&mut arg.clone(), l + 1);
                        (!v.0.is_empty())
                            .then_some(Self::#item(v))
                            .ok_or(Diag {
                                slice: arg.code.cursor..=arg.code.cursor,
                                source: arg.code.source.clone(),
                                error: ErrorType::Any,
                                type_: Construct::#item
                            })
                    }
                } else {
                    quote! {
                        #item::recog(&mut arg.clone(), l + 1).map(Self::#item)
                    }
                }
            };

            let (first, other) = items.split_first().unwrap();
            (item_recog(first), other.iter().map(item_recog))
        };

        quote! {
            #first
                .map_err(|diag| {
                    (diag.slice.clone(), diag.source.clone(), vec![diag.clone()], diag.type_)
                })
            #(
                .or_else(|(slice, source, mut diags, type_)| {
                    #other
                        .map_err(|diag| {
                            diags.push(diag.clone());
                            (slice, source, diags, type_)
                        })
                })
            )*
            .map_err(|(slice, source, diags, type_)| {
                Diag {
                    slice,
                    source,
                    error: ErrorType::#name(diags),
                    type_
                }
            })
        }
    };

    quote! {
        #[derive(Clone, Debug, PartialEq)]
        pub enum #name {
            #( #items(#items) ),*
        }

        impl CommonTypes for #name {
            const CONST: Construct = Construct::#name;
        }

        impl Recog for #name {
            fn parse2(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                EnumRecog::parse(arg, l)
            }
        }

        impl CacheCheck for #name {
            const PREFIX: &str = "enum";

            fn unwrap_item(item: ConstructItem) -> Self {
                let ConstructItem::#name(v) = item else {unreachable!()};
                v
            }
        }

        impl EnumRecog for #name {
            fn parse(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                arg.print.print_colored(arg.get_head("enum", Self::CONST, arg.code.cursor), l);
                Self::consume_parse(arg, l)
                    .map(|v| {
                        arg.print.pass_or_fail::<true>(l);
                        v
                    })
                    .map_err(|mut e| {
                        arg.print.pass_or_fail::<false>(l);
                        e.type_ = Construct::#name;
                        e
                    })
            }

            // есть необходимость в `consume` ведь мы делаем `arg.clone`
            fn consume_parse(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                Self::after_debug(arg, l)
                    .map(|v| {
                        arg.code.cursor = v.slice().end() + 1;
                        v
                    })
            }

            // enum не кешируется, потому что:
            // 1. состоит из токенов и конструкций, которые кешируются
            // 2. enum состоит из вариций, если кешировать одну это значит кешировать любую другую
            // fn cache_parse(arg: &mut ParseArgs) -> Self::Output
            fn after_debug(arg: &ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                #tmp
            }
        }

        impl Slicable for #name {
            fn slice(&self) -> Slice {
                match self {
                    #( Self::#items(v) => v.slice() ),*
                }
            }
        }

        paste! {
            #[derive(Clone, Debug, PartialEq)]
            enum [<#name Error>] {
                #( #items([<#items Error>]) ),*
            }
        }
    }
}
