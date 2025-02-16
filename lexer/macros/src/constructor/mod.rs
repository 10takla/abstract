mod common;
mod constructs;
mod enums;
mod items;
mod tokens;

use common::common;
use constructs::{construct_recognize, construct_tokens, constructs, constructs_reckog};
use enums::{enum_recognize, enum_tokens, enums};
use items::{items, items_recognize};
use proc_macro::TokenStream;
use proc_macro2::{token_stream::IntoIter, Group, Literal, TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens};
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    hash::Hash,
    iter::Peekable,
    panic::{catch_unwind, AssertUnwindSafe, UnwindSafe},
    sync::LazyLock,
};
use syn::{
    parse, parse2, parse_macro_input, punctuated::Punctuated, Attribute, Data, DataEnum,
    DataStruct, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, ItemEnum, LitStr,
    Meta, MetaList, Token, Type, Variant,
};
use tokens::{token_recognize, tokens};

pub fn constructor(input: TokenStream) -> TokenStream {
    let mut iter = TokenStream2::from(input).into_iter().peekable();

    const K: [&str; 5] = ["tokens", "enums", "constructs", "items", "common"];
    let mut map: HashMap<String, TokenStream2> = HashMap::new();
    while let Some(item) = iter.next() {
        let TokenTree::Ident(key) = item else {
            panic!("expect ident, find {item:?}");
        };
        let key = key.to_string();
        if !K.contains(&key.as_str()) {
            panic!("One of {K:?}");
        };

        let v = fast_group(&mut iter).unwrap().stream();
        if let Some(val) = map.get_mut(&key) {
            val.extend(v);
        } else {
            map.insert(key, v);
        }
    }

    let tokens_iter = map
        .get("tokens")
        .cloned()
        .unwrap_or(TokenStream2::new())
        .into_iter();
    let constructs_iter = map
        .get("constructs")
        .cloned()
        .unwrap_or(TokenStream2::new())
        .into_iter();
    let enums_iter = map
        .get("enums")
        .cloned()
        .unwrap_or(TokenStream2::new())
        .into_iter();
    let items_iter = map
        .get("items")
        .cloned()
        .unwrap_or(TokenStream2::new())
        .into_iter();
    let common_iter = map
        .get("common")
        .cloned()
        .unwrap_or(TokenStream2::new())
        .into_iter();

    let (mut gt_tokens, mut tokens, mut errors) = tokens(tokens_iter);
    let (mut gt_items, mut items) = items(items_iter);

    let mut enums = enums(enums_iter);
    let ad_constructs = com(
        common_iter,
        (&mut gt_tokens, &mut tokens),
        (&mut gt_items, &mut items),
        &mut errors,
        &mut enums,
    );

    let (gt_enums, enum_names) = {
        let mut enum_names = vec![];
        (
            enums
                .iter()
                .map(|(name, v)| {
                    enum_names.push(name.clone());
                    enum_tokens(name, v, &items) 
                })
                .collect::<TokenStream2>(),
            enum_names,
        )
    };

    let (gt_constructs, constructs) = {
        let t_constructs = ad_constructs
            .iter()
            .map(|v| construct_tokens(&v.0, &items, v.1.clone()));

        let (mut gt_constructs, mut constructs) = constructs(constructs_iter, &items);
        gt_constructs.extend(t_constructs);
        constructs.extend(ad_constructs.into_iter().map(|(v, _)| v));

        (gt_constructs, constructs)
    };

    let common = common(errors, [tokens.into_iter().collect::<Vec<_>>(), enum_names, items, constructs]);
    quote! {
        #gt_tokens
        #gt_enums
        #gt_constructs
        #gt_items

        #common
    }
    .into()
}

fn com(
    mut iter: IntoIter,
    (t_tokens, tokens): (&mut TokenStream2, &mut Vec<Ident>),
    (t_items, items): (&mut TokenStream2, &mut Vec<Ident>),
    errors: &mut HashMap<Ident, Vec<Ident>>,
    enums: &mut Vec<(Ident, Vec<Ident>)>,
) -> Vec<(Ident, Vec<(Ident, bool, Option<Ident>)>)> {
    let mut constructs = vec![];

    while let Some(_) = iter.clone().next() {
        {
            tmp3(&mut iter, token_recognize).map(|(a, b, c)| {
                t_tokens.extend(a);
                tokens.push(b.clone());
                c.map(|v| errors.insert(b, v));
            })
        }
        .or_else(|_| {
            tmp3(&mut iter, enum_recognize).map(|v| {
                enums.push(v);
            })
        })
        .or_else(|_| {
            tmp3(&mut iter, construct_recognize).map(|v| {
                constructs.push(v);
            })
        })
        .or_else(|_| {
            tmp3(&mut iter, items_recognize).map(|(a, b)| {
                t_items.extend(a);
                items.push(b);
            })
        })
        .unwrap();
    }
    constructs
}

pub fn tmp3<T, O: Fn(&mut Peekable<IntoIter>) -> (T, usize)>(
    iter: &mut IntoIter,
    fn_: O,
) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(|| fn_(&mut iter.clone().peekable())))
        .map(|(v, count)| {
            for _ in 0..count {
                iter.next().unwrap();
            }
            v
        })
        .map_err(tmp5)
}

fn fast_ident(item: &TokenTree) -> Result<Ident, String> {
    match item {
        TokenTree::Ident(v) => Ok(v.clone()),
        _ => Err(format!("expect ident, find {item:?}")),
    }
}

fn fast_ident2(iter: &mut Peekable<IntoIter>) -> Result<Ident, String> {
    match out_of_bound(iter)? {
        TokenTree::Ident(v) => {
            iter.next();
            Ok(v)
        }
        item => Err(format!("expect ident, find {item:?}")),
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

fn tmp(res: Result<(), Box<dyn Any + Send>>) -> Result<(), String> {
    res.map_err(|v| tmp5(v))
}

fn tmp5(v: Box<dyn Any + Send>) -> String {
    v.downcast_ref()
        .cloned()
        .or_else(|| v.downcast_ref::<&str>().map(|v| v.to_string()))
        .unwrap()
}
