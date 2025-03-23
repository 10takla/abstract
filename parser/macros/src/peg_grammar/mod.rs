mod parsing;
// mod new;

use parsing::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use quote::{quote, ToTokens};
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

/// Есть 2 типа:
///     Idented - те, что проименованы типом структуры, которая была определны вне выражения peg.
///         Это может быть:
///             Ident - напрямую связаны с определением структуры вне выражения
///             Enum - тип зависит от структуры enum, поэтому необходимо определить enum и проименовать его
///             TokenMarker - маркер уникального токена, значит должен иметь собственную структуру
///     Expretioned - те что могут быть выражeны, через общие структуры. Являются рекурсивной оболочкой над Expretioned до конченой оболочки над Idented
///         В нее не входят:
///             Token (Token<T>) - структура типа, содержит проименованный тип маркера
///             Seq ((T1, .., T2)) - общий тип - tuple
///             Все Wrapped - Rep, Optи т.д.
///
/// Выражение A ::= (B+ C) / "abc" переходит в
///
/// struct Token0Marker => "abc";
///
/// enum A {
///     V0((OneOrZero<B>, C)),
///     V1(Token<Token0Marker>)
/// }
///

fn exprs(
    v: Exprs,
    output: &mut proc_macro2::TokenStream,
    [token_marker_count, enum_count]: [&mut i32; 2],
    maps: &mut (HashMap<String, Ident>, HashMap<Vec<String>, Ident>),
    mut use_name: Option<Ident>,
) -> proc_macro2::TokenStream {
    if let Exprs::Sequence(..) = v {
        use_name = None;
    }

    let mut expr = |v| {
        let mut expr_other = |v| {
            let mut expr_token = |v| match v {
                ExprToken::Ident(v) => v.to_token_stream(),
                ExprToken::Literal(reg) => {
                    let ident = maps.0.get(&reg.to_string()).cloned().unwrap_or_else(|| {
                        let ident = Ident::new(
                            &format!("Token{token_marker_count}Marker"),
                            Span::call_site(),
                        );
                        *token_marker_count += 1;
                        output.extend(quote! {
                            paste::paste! {
                                #[derive(macros::RegularToken, Clone, Debug, PartialEq)]
                                #[reg_expr = #reg]
                                pub struct #ident;
                            }
                        });
                        maps.0.insert(reg.to_string(), ident.clone());
                        ident
                    });
                    quote! {
                        paste::paste! {Token<#ident>}
                    }
                }
                ExprToken::Group(v) => {
                    exprs(v, output, [token_marker_count, enum_count], maps, None)
                }
            };
            match v {
                ExprOther::ExprToken(v) => expr_token(v),
                _ => match v {
                    ExprOther::Reps(v, a) => {
                        let v = expr_token(*v);
                        match a {
                            RepsBehaviorStruct(reps, RepsBehavior::Base) => match reps {
                                Reps::ZeroOrMore => {
                                    quote! {ZeroOrMore<#v>}
                                }
                                Reps::OneOrMore => {
                                    quote! {OneOrMore<#v>}
                                }
                            },
                            RepsBehaviorStruct(reps, RepsBehavior::BreakWhile(a)) => {
                                let a = expr_token(a);
                                match reps {
                                    Reps::ZeroOrMore => {
                                        quote! {ZeroOrMore<#v, BreakWhile<#a>>}
                                    }
                                    Reps::OneOrMore => {
                                        quote! {OneOrMore<#v, BreakWhile<#a>>}
                                    }
                                }
                            }
                        }
                    }
                    ExprOther::Wrapped(v) => match v {
                        Wrapped::NotPredicate(v) => {
                            let v = expr_token(v);
                            quote! {NotPredicate<#v>}
                        }
                        Wrapped::Optional(v) => {
                            let v = expr_token(v);
                            quote! {Option<#v>}
                        }
                        Wrapped::AndPredicate(v) => {
                            let v = expr_token(v);
                            quote! {AndPredicate<#v>}
                        }
                        Wrapped::Cachable(v) => {
                            let v = expr_token(v);
                            quote! {Cachable<#v>}
                        }
                    },

                    ExprOther::ExprToken(..) => unreachable!(),
                },
            }
        };

        match v {
            Expr::OrderingChoice(v) => {
                let i = (0..v.len()).map(Literal::usize_unsuffixed);
                let v = v.clone().into_iter().map(expr_other).collect::<Vec<_>>();

                let tmp = |ident| {
                    output.extend(quote! {
                        paste::paste! {
                            #[derive(macros::Spanable, macros::EnumRecog, Clone, Debug, PartialEq)]
                            pub enum #ident {
                                #(
                                    #[ty(#v)]
                                    [<V #i>] ( <#v as CommonRecog>::Output )
                                ),*
                            }
                        }
                    });
                    ident
                };

                let key = v.iter().map(ToString::to_string).collect();
                if let Some(v) = use_name.clone() {
                    maps.1.insert(key, v.clone());
                    tmp(v)
                } else {
                    maps.1.get(&key).cloned().unwrap_or_else(|| {
                        let ident = Ident::new(&format!("Enum{enum_count}"), Span::call_site());
                        *enum_count += 1;
                        maps.1.insert(key, ident.clone());
                        tmp(ident)
                    })
                }
                .to_token_stream()
            }
            Expr::ExprOther(v) => expr_other(v),
        }
    };

    match v {
        Exprs::Sequence(v) => {
            let v = v.into_iter().map(expr);
            quote! {(#(#v),*)}
        }
        Exprs::Expr(v) => expr(*v),
    }
}

pub fn peg_grammar(input: TokenStream) -> TokenStream {
    let items = syn::parse::<Formulas>(input).unwrap();

    let mut output = proc_macro2::TokenStream::new();
    let [a, b] = [&mut 0, &mut 0];
    let map = &mut Default::default();

    for Formula {
        is_cachable,
        name,
        exprs: v,
    } in items.0
    {
        let v1 = exprs(v.clone(), &mut output, [a, b], map, Some(name.clone()));
        if let Exprs::Expr(v) = v
            && let Expr::OrderingChoice(..) = *v
        {
        } else {
            output.extend(if is_cachable {
                quote! {pub type #name = Cachable<#v1>;}
            } else {
                quote! {pub type #name = #v1;}
            });
        }
    }
    // panic!("{}", output.to_string());
    output.into()
}
