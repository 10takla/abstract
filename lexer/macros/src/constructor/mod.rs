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
    collections::HashMap,
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
    let ((mut gt_tokens, mut tokens, mut errors), (mut gt_items, mut items)) = (
        tokens(map.get("tokens").unwrap().clone().into_iter()),
        items(map.get("items").unwrap().clone().into_iter()),
    );

    let ([(t_tokens, ad_tokens), (t_items, ad_items)], ad_errors, ad_enums, ad_constructs) =
        com(map.get("common").unwrap().clone().into_iter());

    gt_tokens.extend(t_tokens);
    tokens.extend(ad_tokens);
    errors.extend(ad_errors);

    gt_items.extend(t_items);
    items.extend(ad_items);

    let (gt_enums, enums) = {
        let mut v = enums(map.get("constructs").unwrap().clone().into_iter());
        v.extend(ad_enums);
        let mut enums = vec![];
        (
            v.iter()
                .map(|v| {
                    enums.push(v.0.clone());
                    enum_tokens(&v.0, &v.1, &items)
                })
                .collect::<TokenStream2>(),
            enums,
        )
    };

    let (gt_constructs, constructs) = {
        let t_constructs = ad_constructs
            .iter()
            .map(|v| construct_tokens(&v.0, &items, v.1.clone()));

        let (mut gt_constructs, mut constructs) =
            constructs(map.get("constructs").unwrap().clone().into_iter(), &items);
        gt_constructs.extend(t_constructs);
        constructs.extend(ad_constructs.into_iter().map(|(v, _)| v));

        (gt_constructs, constructs)
    };

    let common = common(errors, [tokens, enums, items, constructs]);
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
) -> (
    [(TokenStream2, Vec<Ident>); 2],
    HashMap<Ident, Vec<Ident>>,
    Vec<(Ident, Vec<Ident>)>,
    Vec<(Ident, Vec<(Ident, bool, Option<Ident>)>)>,
) {
    let [mut tokens, mut items]: [Vec<Ident>; 2] = Default::default();
    let [mut t_tokens, mut t_items]: [TokenStream2; 2] = Default::default();
    let mut constructs = vec![];
    let mut enums = vec![];
    let mut errors = HashMap::new();

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
    (
        [(t_tokens, tokens), (t_items, items)],
        errors,
        enums,
        constructs,
    )
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
