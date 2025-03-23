use super::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use quote::quote;
use std::{
    collections::{HashMap, VecDeque},
    iter::{from_fn, once},
};
use syn::{
    parse::{discouraged::Speculative, Parse},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Paren},
    Token,
};

fn exprs2(v: Exprs, [token_count, enum_count, reps_count, seq_count]: [&mut i32; 4]) -> ItemType {
    let mut expr = |v| {
        let mut expr_other = |v| {
            let tmp_reps_count = *reps_count;
            *reps_count += 1;
            let mut expr_token = |v| match v {
                ExprToken::Ident(v) => ItemType::Literal(v),
                ExprToken::Literal(reg) => {
                    maps.0.get(&reg.to_string()).cloned().unwrap_or_else(|| {
                        let ident = Ident::new(&format!("Token{token_count}"), Span::call_site());
                        output.extend(quote! {
                            paste::paste! {
                                type #ident = Token<[<#ident Marker>]>;
                                #[derive(macros::RegularToken, Clone, Debug, PartialEq)]
                                #[reg_expr = #reg]
                                pub struct [<#ident Marker>];
                            }
                        });
                        *token_count += 1;
                        maps.0.insert(reg.to_string(), ident.clone());
                        ident
                    })
                }
                ExprToken::Group(v) => exprs(
                    v,
                    output,
                    [token_count, enum_count, reps_count, seq_count],
                    maps,
                ),
            };
            match v {
                ExprOther::ExprToken(v) => expr_token(v),
                _ => {
                    let v = match v {
                        ExprOther::Reps(v, a) => {
                            let v = expr_token(*v);
                            match a {
                                RepsBehavior::Base(a) => match a {
                                    Reps::ZeroOrMore => {
                                        quote! {ZeroOrMore<#v>}
                                    }
                                    Reps::OneOrMore => {
                                        quote! {OneOrMore<#v>}
                                    }
                                },
                                RepsBehavior::BreakWhile(a, c) => {
                                    let c = expr_token(c);
                                    match a {
                                        Reps::ZeroOrMore => {
                                            quote! {ZeroOrMore<#v, BreakWhile<#c>>}
                                        }
                                        Reps::OneOrMore => {
                                            quote! {OneOrMore<#v, BreakWhile<#c>>}
                                        }
                                    }
                                }
                            }
                        }
                        ExprOther::NotPredicate(v) => {
                            let v = expr_token(v);
                            quote! {NotPredicate<#v>}
                        }
                        ExprOther::Optional(v) => {
                            let v = expr_token(v);
                            quote! {Option<#v>}
                        }
                        ExprOther::AndPredicate(v) => {
                            let v = expr_token(v);
                            quote! {AndPredicate<#v>}
                        }
                        ExprOther::Cachable(v) => {
                            let v = expr_token(v);
                            quote! {Cachable<#v>}
                        }
                        ExprOther::ExprToken(..) => unreachable!(),
                    };

                    let key = v.to_string();
                    maps.3.get(&key).cloned().unwrap_or_else(|| {
                        let ident = Ident::new(&format!("Rep{tmp_reps_count}"), Span::call_site());
                        maps.3.insert(key, ident.clone());
                        output.extend(quote! {
                            type #ident = #v;
                        });
                        ident
                    })
                }
            }
        };

        match v {
            Expr::OrderingChoice(v) => {
                let i = v
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(i, _)| Literal::usize_unsuffixed(i))
                    .collect::<Vec<_>>();
                let v = v.into_iter().map(expr_other).collect::<Vec<_>>();

                let key = v.iter().map(ToString::to_string).collect();
                maps.1.get(&key).cloned().unwrap_or_else(|| {
                    let ident = Ident::new(&format!("Enum{enum_count}"), Span::call_site());
                    *enum_count += 1;
                    maps.1.insert(key, ident.clone());
                    {
                        let v = quote! {
                            paste::paste! {
                                #[derive(macros::Spanable, macros::EnumRecog, Clone, Debug, PartialEq)]
                                pub enum #ident {
                                    #(
                                        #[ty(#v)]
                                        [<V #i>] ( <#v as CommonRecog>::Output )
                                    ),*
                                }
                            }
                        };
                        output.extend(v);
                    }
                    ident
                })
            }
            Expr::ExprOther(v) => expr_other(v),
        }
    };

    match v {
        Exprs::Sequence(v) => {
            let v = v.into_iter().map(|v| expr(v)).collect::<Vec<_>>();
            let key = v.iter().map(ToString::to_string).collect();
            maps.1.get(&key).cloned().unwrap_or_else(|| {
                let ident = Ident::new(&format!("Seq{seq_count}"), Span::call_site());
                maps.1.insert(key, ident.clone());
                output.extend(quote! {
                    type #ident = (#(#v),*);
                });
                *seq_count += 1;
                ident
            })
        }
        Exprs::Expr(v) => expr(*v),
    }
}

// impl ExprToken {
//     fn iter(self) -> R {
//         match self {
//             ExprToken::Group(v) => v.iter(),
//             ExprToken::Ident(v) => R::Once(ItemType::Ident(v)),
//             ExprToken::Literal(v) => R::Once(ItemType::Literal(v)),
//         }
//     }
// }

// impl ExprOther {
//     fn iter(self) -> R {
//         match self {
//             ExprOther::Reps(v, reps_behavior) => {
//                 let mut iter = v.iter();

//                 let reps = match reps_behavior {
//                     RepsBehavior::Base(v) => v,
//                     RepsBehavior::BreakWhile(v, ..) => v,
//                 };

//                 let beh = match reps_behavior {
//                     RepsBehavior::Base(..) => RepsBehavior2::Base,
//                     RepsBehavior::BreakWhile(_, v) => RepsBehavior2::BreakWhile(v.iter()),
//                 };

//                 Box::new(iter.chain(once(ItemType::Wrapped(Wrapped::Reps()))))
//             }
//             ExprOther::Optional(v) => R::Wrapped(Box::new(
//                 v.iter()
//                     .chain(R::once once(ItemType::Wrapped(Wrapped::NotPredicate))),
//             )),
//             ExprOther::AndPredicate(v) => {
//                 Box::new(v.iter().chain(once(ItemType::Wrapped(Wrapped::Option))))
//             }
//             ExprOther::NotPredicate(v) => Box::new(
//                 v.iter()
//                     .chain(once(ItemType::Wrapped(Wrapped::AndPredicate))),
//             ),
//             ExprOther::Cachable(v) => {
//                 Box::new(v.iter().chain(once(ItemType::Wrapped(Wrapped::Cachable))))
//             }
//             ExprOther::ExprToken(v) => v.iter(),
//         }
//     }
// }

// enum R {
//     Wrapped(Box<dyn Iterator<Item = ItemType>>, ItemType),
//     Once(ItemType),
// }

// impl Expr {
//     fn iter(self) -> R {
//         match self {
//             Expr::OrderingChoice(v) => R::Wrapped(
//                 Box::new(v.into_iter().flat_map(ExprOther::iter)),
//                 ItemType::OrderingChoice,
//             ),
//             Expr::ExprOther(v) => R::Once(Box::new(v.iter())),
//         }
//     }
// }

// impl Exprs {
//     fn iter(self) -> Box<dyn Iterator<Item = R>> {
//         match self {
//             Exprs::Sequence(v) => {
//                 R::Wrapped(Box::new(v.into_iter().flat_map(Expr::iter)), ItemType::Seq)
//             }
//             Exprs::Expr(v) => R::Once(Box::new(v.iter())),
//         }
//     }
// }

// pub fn peg_grammar(input: TokenStream) -> TokenStream {
//     let Formulas(items) = parse_macro_input!(input);

//     let mut output = proc_macro2::TokenStream::new();
//     let [token_count, enum_count, reps_count, seq_count] = [&mut 0, &mut 0, &mut 0, &mut 0];
//     let maps = &mut Default::default();

//     struct Item {
//         is_cachable: bool,
//         name: Ident,
//         item: ItemType,
//     }
//     enum K {
//         Many(VecDeque<Ident>),
//         One(Ident)
//     }
//     let mut iter = items.into_iter();
//     let mut inner: Box<dyn Iterator<Item = _>> = Box::new(std::iter::empty::<_>());
//     let mut items = vec![];
//     let mut name_stack = Vec::new();

//     from_fn(|| {
//         inner.next().or_else(|| {
//             let Formula {
//                 is_cachable,
//                 name,
//                 exprs: v,
//             } = iter.next()?;
//             inner = Box::new(
//                 v.iter()
//                     .map(|(a, item)| {
//                         (a, Item {
//                             is_cachable,
//                             name,
//                             item,
//                         })
//                     })
//             );
//             inner.next()
//         })
//     })
//     .map(
//         |(name_stack,
//         Item {
//              name: glob_name,
//              is_cachable,
//              item,
//         })| {
//             match item {
//                 ItemType::Literal(reg) => {
//                     let name = Ident::new(&format!("Token{token_count}"), Span::call_site());
//                     *token_count += 1;
//                     output.extend(quote! {
//                         paste::paste! {
//                             type #name = Token<[<#name Marker>]>;
//                             #[derive(macros::RegularToken, Clone, Debug, PartialEq)]
//                             #[reg_expr = #reg]
//                             pub struct [<#name Marker>];
//                         }
//                     });
//                     name_stack.push_back(name);
//                 }
//                 ItemType::Ident(name) => {
//                     name_stack.push_back(name);
//                 }
//                 ItemType::Wrapped(v) => {
//                     assert!(name_stack.is_empty());
//                     let item = name_stack.pop_front().unwrap();

//                     let v = match v {
//                         Wrapped::NotPredicate => quote! {NotPredicate<#item>},
//                         Wrapped::Option => quote! {Option<#item>},
//                         Wrapped::AndPredicate => quote! {AndPredicate<#item>},
//                         Wrapped::Cachable => quote! {Cachable<#item>},
//                         Wrapped::Reps(a, b) => match a {
//                             RepsBehavior::Base(a) => match a {
//                                 Reps::ZeroOrMore => {
//                                     quote! {ZeroOrMore<#item>}
//                                 }
//                                 Reps::OneOrMore => {
//                                     quote! {OneOrMore<#item>}
//                                 }
//                             },
//                             RepsBehavior::BreakWhile(a, c) => {
//                                 let c = c.iter();
//                                 match a {
//                                     Reps::ZeroOrMore => {
//                                         quote! {ZeroOrMore<#item, BreakWhile<#c>>}
//                                     }
//                                     Reps::OneOrMore => {
//                                         quote! {OneOrMore<#item, BreakWhile<#c>>}
//                                     }
//                                 }
//                             }
//                         },
//                     };

//                     let name = Ident::new(&format!("Token{reps_count}"), Span::call_site());
//                     *reps_count += 1;

//                     output.extend(quote! {
//                         type #name = #v
//                     });

//                     name_stack.push_front(name);
//                 }
//                 ItemType::Seq => {
//                     let name = Ident::new(&format!("Seq{seq_count}"), Span::call_site());
//                     *seq_count += 1;

//                     assert!(name_stack.is_empty());

//                     let items = name_stack.iter();
//                     name_stack.clear();

//                     output.extend(quote! {
//                         type #name = (#(#items),*);
//                     });

//                     name_stack.push_front(name);
//                 }
//                 ItemType::OrderingChoice => {
//                     let name = Ident::new(&format!("Enum{enum_count}"), Span::call_site());
//                     *enum_count += 1;

//                     assert!(name_stack.is_empty());

//                     let i = (0..name_stack.len()).map(Literal::usize_unsuffixed);

//                     let items = name_stack.iter();
//                     name_stack.clear();

//                     output.extend(quote! {
//                         paste::paste! {
//                             #[derive(macros::Spanable, macros::EnumRecog, Clone, Debug, PartialEq)]
//                             pub enum #name {
//                                 #(
//                                     #[ty(#items)]
//                                     [<V #i>] ( <#items as CommonRecog>::Output )
//                                 ),*
//                             }
//                         }
//                     });
//                     name_stack.push_front(name);
//                 }
//             }
//         },
//     )
//     .collect::<Vec<_>>();

//     // panic!("{}", output.to_string());
//     output.into()
// }

enum Wrapped {
    NotPredicate,
    Option,
    AndPredicate,
    Cachable,
    Reps(Reps, RepsBehavior2),
}

enum RepsBehavior2 {
    Base,
    BreakWhile,
}

type IteratorType = Box<dyn Iterator<Item = ItemType>>;

enum ItemType {
    Ident(Ident),
    Literal(Literal),
    Wrapped(Wrapped),
    Seq,
    OrderingChoice,
}