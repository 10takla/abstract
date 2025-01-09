use super::{check_pass_fail, fast_group, fast_ident, fast_ident2, tmp, tmp3, tmp5, COMMON};
use proc_macro2::{token_stream::IntoIter, Group, Ident, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::{
    collections::HashMap,
    fmt::Debug,
    iter::Peekable,
    panic::{catch_unwind, AssertUnwindSafe},
};
use syn::{parse2, LitStr};

pub fn tokens(mut iter: IntoIter) -> (TokenStream2, Vec<Ident>, HashMap<Ident, Vec<Ident>>) {
    let (mut tokens, mut names, mut errors): (
        TokenStream2,
        Vec<Ident>,
        HashMap<Ident, Vec<Ident>>,
    ) = Default::default();

    while iter.clone().next().is_some() {
        let v = tmp3(&mut iter, token_recognize).unwrap();
        tokens.extend(v.0);
        names.push(v.1.clone());
        v.2.map(|t| errors.insert(v.1, t));
    }
    
    (tokens, names, errors)
}

pub fn token_recognize(
    iter: &mut Peekable<IntoIter>,
) -> ((TokenStream2, Ident, Option<Vec<Ident>>), usize) {
    let mut counter = 0;

    let name = fast_ident2(iter).unwrap();
    counter += 1;

    let mut errors = None;

    let body = match iter.next().unwrap() {
        TokenTree::Literal(v) => {
            parse2::<LitStr>(TokenTree::from(v.clone()).into()).unwrap();
            counter += 1;
            quote! {
                reg_observe(arg, #v).map_err(|v| (v..=v, ErrorType::Reg(#v)))
            }
        }
        TokenTree::Group(body) => {
            let e = fast_group(iter).unwrap();
            counter += 1;
            errors = Some(
                body.stream()
                    .into_iter()
                    .map(|v| fast_ident(&v).unwrap())
                    .collect(),
            );
            counter += 1;
            quote! {
                (|arg: &ParseArgs| #e)(arg)
            }
        }
        _ => unreachable!(),
    };

    let common = &*COMMON;
    let check_pass_fail = check_pass_fail("token", &name);
    let tokens = quote! {
        #[derive(Clone, Debug, PartialEq)]
        pub struct #name(Slice);
        impl CommonTypes for #name {
            const CONST: Construct = Construct::#name;
        }
        impl #name {
            #common

            #check_pass_fail

            fn parse(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                let pos = arg.code.cursor;
                let mut tmp = |arg: &mut ParseArgs, v| {
                    arg.print.print_colored(format!("{v} {}", arg.get_head("token", Self::CONST, pos)), l);
                };

                Self::consume_parse(arg)
                .map(|v| {
                    tmp(arg, format!("{}", tmp_pass_or_fail::<true>()));
                    v
                }).map_err(|e| {
                    tmp(arg, format!("{}({})", tmp_pass_or_fail::<false>(), e.end()));
                    e
                })
            }

            fn consume_parse(arg: &mut ParseArgs) -> <Self as CommonTypes>::Output {
                Self::after_debug(arg)
                .map(|v| {
                    arg.code.cursor = v.slice().end() + 1;
                    v
                })
            }

            fn after_debug(arg: &ParseArgs) -> <Self as CommonTypes>::Output {
                #body.map(Self).map_err(|(slice, error)| Diag {
                    slice,
                    source: arg.code.source.clone(),
                    error,
                    type_: Construct::#name
                })
            }
        }

        impl Slicable for #name {
            fn slice(&self) -> Slice {
                self.0.clone()
            }
        }
    };

    ((tokens, name, errors), counter)
}
