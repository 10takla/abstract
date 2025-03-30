use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use quote::{quote, ToTokens};
use std::{
    collections::{HashMap, VecDeque},
    iter::{from_fn, once},
};
use syn::{
    custom_keyword,
    ext::IdentExt,
    parenthesized,
    parse::{discouraged::Speculative, Parse},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Paren},
    Error, Token,
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
    pub head: FormulaHead,
    pub exprs: Exprs,
}

#[derive(Clone, Debug)]
pub struct FormulaHead {
    pub is_cachable: bool,
    pub is_wrapped: bool,
    pub is_error: bool,
    pub name: Ident,
}

mod kw {
    use syn::custom_keyword;
    custom_keyword!(error);
}

impl Parse for FormulaHead {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            is_cachable: <Token![@]>::parse(input).is_ok(),
            is_error: if input.peek(kw::error) {
                input.parse::<kw::error>()?;
                true
            } else {
                false
            },
            name: Parse::parse(input)?,
            is_wrapped: {
                let v = input.peek(Paren) && {
                    let content;
                    parenthesized!(content in input);
                    content.is_empty()
                };
                LeftArrow::parse(input)?;
                v
            },
        })
    }
}

impl Parse for Formula {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            head: FormulaHead::parse(input)?,
            exprs: Parse::parse(input)?,
        })
    }
}

pub struct LeftArrow;
impl Parse for LeftArrow {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![::]) {
            <Token![::]>::parse(input)?;
            <Token![=]>::parse(input)?;
        } else if !(<Token![->]>::parse(input).is_ok() || <Token![<-]>::parse(input).is_ok()) {
            return Err(Error::new(input.span(), "expexct ::= | -> | <-"));
        }
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
                if FormulaHead::parse(&lookahead).is_ok() {
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
    Ident {
        is_boxed: bool,
        ident: Ident,
    },
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
            let is_boxed = <Token![$]>::parse(input).is_ok();
            Parse::parse(input)
                .map(|ident| Self::Ident { is_boxed, ident })
                .or_else(|_| Parse::parse(input).map(Self::Literal))
        }
    }
}
