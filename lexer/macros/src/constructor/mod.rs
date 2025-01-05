mod common;
mod constructs;
mod enums;
mod items;
mod tokens;

use common::common;
use constructs::constructs;
use enums::enums;
use items::items;
use proc_macro::TokenStream;
use proc_macro2::{token_stream::IntoIter, Group, Literal, TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens};
use std::{collections::HashMap, hash::Hash, iter::Peekable, sync::LazyLock};
use syn::{
    parse, parse2, parse_macro_input, punctuated::Punctuated, Attribute, Data, DataEnum,
    DataStruct, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, ItemEnum, LitStr,
    Meta, MetaList, Token, Type, Variant,
};
use tokens::tokens;

pub fn constructor(input: TokenStream) -> TokenStream {
    let mut iter = TokenStream2::from(input).into_iter().peekable();

    const K: [&str; 4] = ["tokens", "enums", "constructs", "items"];
    let mut map = HashMap::new();
    while let Some(item) = iter.next() {
        let TokenTree::Ident(key) = item else {
            panic!("expect ident, find {item:?}");
        };
        let key = key.to_string();
        if !K.contains(&key.as_str()) {
            panic!("One of {K:?}");
        };
        if map.contains_key(&key) {
            panic!("field {key} already filled");
        }
        map.insert(key, fast_group(&mut iter).unwrap());
    }
    let (t1, tokens, errors) = tokens(map.get("tokens").unwrap());
    let (t2, enums) = enums(map.get("enums").unwrap());
    let (t4, items) = items(map.get("items").unwrap());
    let (t3, constructs) = constructs(map.get("constructs").unwrap(), &items);

    let common = common(errors, [tokens, enums, items, constructs]);
    quote! {
        #t1
        #t2
        #t3
        #t4

        #common
    }
    .into()
}

const COMMON: LazyLock<TokenStream2> = LazyLock::new(|| {
    quote! {
        fn recog(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
            if let Some(e) = Self::check_fail(arg, l) {
                Err(e)
            } else {
                Self::check_pass(arg, l).map(|(_, s)| Ok(s)).unwrap_or_else(|| Self::parse(arg, l))
            }
        }
    }
});

fn check_pass_fail(name: &str, t: impl ToTokens) -> TokenStream2 {
    let literal = Literal::string(name);
    quote! {
        fn check_pass(arg: &mut ParseArgs, l: usize) -> Option<(usize, Self)> {
            let when_not_fail = arg.code.cursor;
            arg.c_a_d.borrow().cache.pass.iter().enumerate().find_map(|(i, k)| {
                k.items.get(k.index).and_then(|v| {
                    (v.0 == Self::CONST && v.1 == when_not_fail).then(|| {
                        let ConstructItem::#t(ref v) = v.2 else {unreachable!()};
                        arg.code.cursor = v.slice().end() + 1;
                        arg.print.from_cache::<true>(#literal, Self::CONST, l);
                        (i, v.clone())
                    })
                })
            })
        }
        fn check_fail(arg: &mut ParseArgs, l: usize) -> Option<<Self as CommonTypes>::Error> {
            arg.c_a_d.borrow().cache.fails.get(&(Self::CONST, arg.code.cursor)).map(|e| {
                arg.print.from_cache::<false>(#literal, Self::CONST, l);
                e
            }).cloned()
        }
    }
}

fn fast_ident(item: &TokenTree) -> Result<Ident, String> {
    match item {
        TokenTree::Ident(v) => Ok(v.clone()),
        _ => Err(format!("expect ident, find {item:?}")),
    }
}

fn fast_group(iter: &mut Peekable<IntoIter>) -> Result<Group, String> {
    match out_of_bound(iter)? {
        TokenTree::Group(v) => {
            iter.next();
            Ok(v)
        }
        item => Err(format!("expect group, find {item:?}")),
    }
}

fn fast_puncts(pat: &str, iter: &mut Peekable<IntoIter>) -> Result<(), String> {
    pat.chars().try_for_each(|v| match out_of_bound(iter)? {
        TokenTree::Punct(a) if a.as_char() == v => {
            iter.next();
            Ok(())
        }
        _ => Err(format!("expect {pat}")),
    })
}

fn out_of_bound(iter: &mut Peekable<IntoIter>) -> Result<TokenTree, String> {
    iter.peek()
        .cloned()
        .ok_or("выход за последний токен".into())
}
