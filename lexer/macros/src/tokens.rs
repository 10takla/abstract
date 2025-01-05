use crate::{check_pass_fail, fast_group, fast_ident, COMMON};
use proc_macro2::{Group, Ident, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::{collections::HashMap, fmt::Debug};
use syn::{parse2, LitStr};

pub fn tokens(
    g: &Group,
) -> (
    TokenStream2,
    Vec<Ident>,
    HashMap<Ident, impl Iterator<Item = Ident> + Clone + Debug>,
) {
    let mut iter = g.stream().into_iter().peekable();
    let mut vec = vec![];
    let mut token_names = vec![];
    let mut errors = HashMap::new();
    while let Some(item) = iter.next() {
        let init_item = fast_ident(&item).unwrap();

        let v = match iter.next().unwrap() {
            TokenTree::Literal(v) => {
                parse2::<LitStr>(TokenTree::from(v.clone()).into()).unwrap();
                quote! {
                    reg_observe(arg, #v).map_err(|v| (v..=v, ErrorType::Reg))
                }
            }
            TokenTree::Group(body) => {
                let e = fast_group(&mut iter).unwrap();
                errors.insert(
                    init_item.clone(),
                    body.stream().into_iter().map(|v| fast_ident(&v).unwrap()),
                );
                quote! {
                    (|arg: &ParseArgs| #e)(arg)
                }
            }
            _ => unreachable!(),
        };

        let common = &*COMMON;
        let check_pass_fail = check_pass_fail("token", &init_item);
        vec.push(quote! {
            #[derive(Clone, Debug)]
            pub struct #init_item(Slice);
            impl CommonTypes for #init_item {
                const CONST: Construct = Construct::#init_item;
            }
            impl #init_item {
                #common

                #check_pass_fail

                fn parse(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                    let v2 = arg.code.cursor;
                    let mut tmp = |arg: &mut ParseArgs, v| {
                        arg.print.print_colored(format!("{v} token {:?} {:?} {}", Self::CONST, arg.c_a_d.borrow().cache.pass, v2), l);
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
                    #v.map(Self).map_err(|(slice, error)| Diag {
                        slice,
                        source: arg.code.source.clone(),
                        error
                    })
                }
            }

            impl Slicable for #init_item {
                fn slice(&self) -> Slice {
                    self.0.clone()
                }
            }
        });
        token_names.push(init_item);
    }

    (quote! {#(#vec)*}, token_names, errors)
}
