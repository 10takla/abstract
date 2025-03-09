use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use quote::quote;
use std::collections::HashMap;
use syn::{
    parse::{discouraged::Speculative, Parse},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Paren},
    Token,
};

fn exprs(
    v: Exprs,
    output: &mut proc_macro2::TokenStream,
    [token_count, enum_count, reps_count, seq_count]: [&mut i32; 4],
    maps: &mut (
        HashMap<String, Ident>,
        HashMap<Vec<String>, Ident>,
        HashMap<Vec<String>, Ident>,
        HashMap<String, Ident>,
    ),
) -> Ident {
    let mut expr = |v| {
        let tmp_enum_count = *enum_count;
        *enum_count += 1;

        let mut expr_other = |v| {
            let tmp_reps_count = *reps_count;
            *reps_count += 1;
            let mut expr_token = |v| match v {
                ExprToken::Ident(v) => v,
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
                        ExprOther::ZeroOrMore(v) => {
                            let v = expr_token(*v);
                            quote! {ErrorBreakRepetition<#v>}
                        }
                        ExprOther::OneOrMore(v) => {
                            let v = expr_token(*v);
                            quote! {OneOrMore<#v>}
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
                    let ident = Ident::new(&format!("Enum{tmp_enum_count}"), Span::call_site());
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

pub fn peg_grammar(input: TokenStream) -> TokenStream {
    let items: Formulas = parse_macro_input!(input);

    let mut output = proc_macro2::TokenStream::new();

    let [a, b, c, d] = [&mut 0, &mut 0, &mut 0, &mut 0];
    let map = &mut Default::default();
    for Formula {
        is_cachable,
        name,
        exprs: v,
    } in items.0
    {
        let v = exprs(v, &mut output, [a, b, c, d], map);
        output.extend(if is_cachable {
            quote! {
                pub type #name = Cachable<#v>;
            }
        } else {
            quote! {
                pub type #name = #v;
            }
        });
    }
    // panic!("{}", output.to_string());
    output.into()
}

struct LeftArrow;
impl Parse for LeftArrow {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        <Token![:]>::parse(input)?;
        <Token![:]>::parse(input)?;
        <Token![=]>::parse(input)?;
        Ok(Self)
    }
}

#[derive(Clone, Debug)]
struct Formulas(Vec<Formula>);

impl Parse for Formulas {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut vec = vec![];
        loop {
            if let Ok(v) = Formula::parse(input) {
                vec.push(v);
            } else {
                break;
            }
        }
        if vec.is_empty() {
            Err(input.error("Expect more 0 items"))
        } else {
            Ok(Self(vec))
        }
    }
}

#[derive(Clone, Debug)]
struct Formula {
    is_cachable: bool,
    name: Ident,
    exprs: Exprs,
}

/// Sequence: e1 e2
// Sequence(Vec<Expr>)
impl Parse for Formula {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            is_cachable: <Token![@]>::parse(input).is_ok(),
            name: Parse::parse(input)?,
            exprs: {
                LeftArrow::parse(input)?;
                Parse::parse(input)?
            },
        })
    }
}

/// https://en.wikipedia.org/wiki/Parsing_expression_grammar#Composite_parsing_expressions
#[derive(Clone, Debug)]
enum Exprs {
    /// Sequence: e1 e2
    Sequence(Vec<Expr>),
    Expr(Box<Expr>),
}

impl Parse for Exprs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut vec = vec![];
        loop {
            if let Ok(v) = Expr::parse(input) {
                vec.push(v);
                let lookahead = input.fork();
                <Token![@]>::parse(&lookahead);
                if Ident::parse(&lookahead)
                    .and_then(|_| LeftArrow::parse(&lookahead))
                    .is_ok()
                {
                    break;
                }
            } else {
                break;
            }
        }
        if vec.is_empty() {
            Err(input.error("Expect more 0 items"))
        } else if vec.len() == 1 {
            Ok(Self::Expr(Box::new(vec[0].clone())))
        } else {
            Ok(Self::Sequence(vec))
        }
    }
}

#[derive(Clone, Debug)]
enum Expr {
    /// Ordered choice: e1 / e2
    OrderingChoice(Punctuated<ExprOther, Token![/]>),
    ExprOther(ExprOther),
}

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut vec = Punctuated::<ExprOther, Token![/]>::parse_separated_nonempty(input)?;
        if vec.is_empty() {
            Err(input.error(""))
        } else if vec.len() == 1 {
            Ok(Self::ExprOther(vec[0].clone()))
        } else {
            Ok(Self::OrderingChoice(vec))
        }
    }
}

#[derive(Clone, Debug)]
enum ExprOther {
    /// Zero-or-more: e*
    ZeroOrMore(Box<ExprToken>),
    /// One-or-more: e+
    OneOrMore(Box<ExprToken>),
    /// Optional: e?
    Optional(ExprToken),
    /// And-predicate: &e
    AndPredicate(ExprToken),
    /// Not-predicate: !e
    NotPredicate(ExprToken),
    /// @
    Cachable(ExprToken),
    ExprToken(ExprToken),
}

impl Parse for ExprOther {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        (|| {
            let lookahead_input = input.fork();
            if let Ok(expr_token) = Parse::parse(&lookahead_input) {
                if lookahead_input.peek(Token![*]) {
                    input.advance_to(&lookahead_input);
                    input.parse::<Token![*]>().unwrap();
                    return Ok(Self::ZeroOrMore(expr_token));
                }
            }
            Err(())
        })()
        .or_else(|_| {
            let lookahead_input = input.fork();
            if let Ok(expr_token) = Parse::parse(&lookahead_input) {
                if lookahead_input.peek(Token![+]) {
                    input.advance_to(&lookahead_input);
                    input.parse::<Token![+]>().unwrap();
                    return Ok(Self::OneOrMore(expr_token));
                }
            }
            Err(())
        })
        .or_else(|_| {
            let lookahead_input = input.fork();
            if let Ok(expr_token) = Parse::parse(&lookahead_input) {
                if lookahead_input.peek(Token![?]) {
                    input.advance_to(&lookahead_input);
                    input.parse::<Token![?]>().unwrap();
                    return Ok(Self::Optional(expr_token));
                }
            }
            Err(())
        })
        .or_else(|_| {
            <Token![&]>::parse(input)?;
            Parse::parse(input).map(Self::AndPredicate)
        })
        .or_else(|_| {
            <Token![!]>::parse(input)?;
            Parse::parse(input).map(Self::NotPredicate)
        })
        .or_else(|_| {
            <Token![@]>::parse(input)?;
            Parse::parse(input).map(Self::Cachable)
        })
        .or_else(|_| Parse::parse(input).map(Self::ExprToken))
    }
}

#[derive(Clone, Debug)]
enum ExprToken {
    /// Group: (e)
    Group(Exprs),
    Ident(Ident),
    Literal(Literal),
}

impl Parse for ExprToken {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Paren) {
            let content;
            syn::parenthesized!(content in input);
            Parse::parse(&content).and_then(|v: Exprs| {
                if let Exprs::Expr(v) = v.clone()
                    && let Expr::ExprOther(ExprOther::ExprToken(..)) = *v
                {
                    Err(input.error("Expect none-single items"))
                } else {
                    Ok(Self::Group(v))
                }
            })
        } else {
            Parse::parse(input)
                .map(Self::Ident)
                .or_else(|_| Parse::parse(input).map(Self::Literal))
        }
    }
}
