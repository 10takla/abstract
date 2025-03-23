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

#[derive(Clone, Debug)]
pub struct Formulas(pub Vec<Formula>);

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
pub struct Formula {
    pub is_cachable: bool,
    pub name: Ident,
    pub exprs: Exprs,
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

pub struct LeftArrow;
impl Parse for LeftArrow {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        <Token![:]>::parse(input)?;
        <Token![:]>::parse(input)?;
        <Token![=]>::parse(input)?;
        Ok(Self)
    }
}

/// https://en.wikipedia.org/wiki/Parsing_expression_grammar#Composite_parsing_expressions
#[derive(Clone, Debug)]
pub enum Exprs {
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
pub enum Expr {
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
pub enum ExprOther {
    Reps(Box<ExprToken>, RepsBehaviorStruct),
    Wrapped(Wrapped),
    ExprToken(ExprToken),
}

impl Parse for ExprOther {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        {
            let lookahead_input = input.fork();
            Parse::parse(&lookahead_input).and_then(|a| {
                Parse::parse(&lookahead_input).map(|b| {
                    input.advance_to(&lookahead_input);
                    Self::Reps(a, b)
                })
            })
        }
        .or_else(|_| Parse::parse(input).map(Self::Wrapped))
        .or_else(|_| Parse::parse(input).map(Self::ExprToken))
    }
}

#[derive(Clone, Debug)]
pub enum Wrapped {
    /// Optional: e?
    Optional(ExprToken),
    /// And-predicate: &e
    AndPredicate(ExprToken),
    /// Not-predicate: !e
    NotPredicate(ExprToken),
    /// @
    Cachable(ExprToken),
}

impl Parse for Wrapped {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        (|| {
            let lookahead_input = input.fork();
            if let Ok(expr_token) = Parse::parse(&lookahead_input) {
                if lookahead_input.peek(Token![?]) {
                    input.advance_to(&lookahead_input);
                    input.parse::<Token![?]>().unwrap();
                    return Ok(Self::Optional(expr_token));
                }
            }
            Err(())
        })()
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
    }
}

#[derive(Clone, Debug)]
pub struct RepsBehaviorStruct(pub Reps, pub RepsBehavior);

#[derive(Clone, Debug)]
pub enum RepsBehavior {
    Base,
    BreakWhile(ExprToken),
}

impl Parse for RepsBehaviorStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Parse::parse(input)
            .map(|v| Self(v, RepsBehavior::Base))
            .or_else(|_| {
                let lookahead_input = input.fork();
                let content;
                syn::bracketed!(content in lookahead_input);
                Parse::parse(&content).and_then(|v| {
                    Parse::parse(&content).map(|n| {
                        input.advance_to(&lookahead_input);
                        Self(v, RepsBehavior::BreakWhile(n))
                    })
                })
            })
    }
}

#[derive(Clone, Debug)]
pub enum Reps {
    // ZeroOrMore(Box<ExprToken>),
    ZeroOrMore,
    // OneOrMore(Box<ExprToken>),
    OneOrMore,
}

impl Parse for Reps {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        <Token![*]>::parse(input)
            .map(|_| Self::ZeroOrMore)
            .or_else(|_| <Token![+]>::parse(input).map(|_| Self::OneOrMore))
    }
}

#[derive(Clone, Debug)]
pub enum ExprToken {
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
