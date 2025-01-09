use super::{check_pass_fail, fast_ident, fast_ident2, fast_puncts, tmp, tmp3, tmp5, COMMON};
use proc_macro2::{token_stream::IntoIter, Group, Ident, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::{
    iter::Peekable,
    panic::{catch_unwind, AssertUnwindSafe},
};
use syn::{custom_punctuation, parse2, LitStr};

pub fn enums(mut iter: IntoIter) -> (TokenStream2, Vec<Ident>) {
    let (mut tokens, mut names): (TokenStream2, Vec<Ident>) = Default::default();

    while iter.clone().next().is_some() {
        let v = tmp3(&mut iter, enum_recognize).unwrap();
        tokens.extend(v.0);
        names.push(v.1.clone());
    }

    (tokens, names)
}

pub fn enum_recognize(iter: &mut Peekable<IntoIter>) -> ((TokenStream2, Ident), usize) {
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

    let common = &*COMMON;
    let check_pass_fail = check_pass_fail("enum", &name);
    let (first, other) = items.split_first().unwrap();

    let tokens = quote! {
        #[derive(Clone, Debug, PartialEq)]
        pub enum #name {
            #( #items(#items) ),*
        }

        impl CommonTypes for #name {
            const CONST: Construct = Construct::#name;
        }
        impl #name {
            #common
            #check_pass_fail

            fn parse(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                arg.print.print_colored(arg.get_head("enum", Self::CONST, arg.code.cursor), l);
                Self::consume_parse(arg, l).map(|v| {
                    arg.print.pass_or_fail::<true>(l);
                    v
                }).map_err(|mut e| {
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
                #first::recog(&mut arg.clone(), l + 1).map(Self::#first)
                #(
                    .or_else(|error| {
                        #other::recog(&mut arg.clone(), l + 1).map(Self::#other)
                        .map_err(|e| {
                            (e.end() > error.end()).then_some(e).unwrap_or(error)
                        })
                    })
                )*
            }
        }

        impl Slicable for #name {
            fn slice(&self) -> Slice {
                match self {
                    #( Self::#items(v) => v.slice() ),*
                }
            }
        }
    };

    ((tokens, name), counter)
}
