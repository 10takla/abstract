use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use quote::quote;
use syn::{
    parse::{discouraged::Speculative, Parse},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Paren},
    Token,
};

pub fn peg_grammar(input: TokenStream) -> TokenStream {
    let items: Formulas = parse_macro_input!(input);

    let mut output = proc_macro2::TokenStream::new();

    fn exprs(
        v: Exprs,
        output: &mut proc_macro2::TokenStream,
        [token_count, enum_count, reps_count, seq_count]: [&mut i32; 4],
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
                        let ident = Ident::new(&format!("Token{token_count}"), Span::call_site());
                        output.extend(quote! {
                            paste! {
                                type #ident = Token<[<#ident Marker>]>;
                                #[derive(RegularToken)]
                                #[reg_expr = #reg]
                                struct [<#ident Marker>];
                            }
                        });
                        *token_count += 1;
                        ident
                    }
                    ExprToken::Group(v) => {
                        exprs(v, output, [token_count, enum_count, reps_count, seq_count])
                    }
                };
                match v {
                    ExprOther::ZeroOrMore(v) => {
                        let ident = Ident::new(&format!("Rep{tmp_reps_count}"), Span::call_site());
                        let v = expr_token(v);
                        output.extend(quote! {
                            type #ident = ErrorBreakRepetition<#v>;
                        });
                        ident
                    }
                    ExprOther::ExprToken(v) => expr_token(v),
                    _ => unreachable!(),
                }
            };

            match v {
                Expr::OrderingChoice(v) => {
                    let i = v
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(i, _)| Literal::usize_unsuffixed(i)).collect::<Vec<_>>();
                    let v = v.into_iter().map(expr_other).collect::<Vec<_>>();

                    let enum_ident =
                        Ident::new(&format!("Enum{tmp_enum_count}"), Span::call_site());
                    {
                        let v = quote! {
                            paste! {
                                #[derive(Spanable, EnumRecog)]
                                enum #enum_ident {
                                    #( 
                                        #[ty(#v)] 
                                        [<V #i>] ( <#v as CommonRecog>::Output ) 
                                    ),*
                                }
                            }
                        };
                        output.extend(v);
                    }

                    enum_ident
                }
                Expr::ExprOther(v) => expr_other(v),
            }
        };

        match v {
            Exprs::Sequence(v) => {
                let v = v.into_iter().map(|v| expr(v)).collect::<Vec<_>>();
                let ident = Ident::new(&format!("Seq{seq_count}"), Span::call_site());
                output.extend(quote! {
                    type #ident = (#(#v),*);
                });
                *seq_count += 1;
                ident
            }
            Exprs::Expr(v) => expr(*v),
        }
    }

    let [a, b, c, d] = [&mut 0, &mut 0, &mut 0, &mut 0];
    for Formula(ident, v) in items.0 {
        let v = exprs(v, &mut output, [a, b, c, d]);
        output.extend(quote! {
            type #ident = #v;
        });
    }
    // panic!("{}", output.to_string());
    output.into()
}

#[derive(Clone, Debug)]
struct Formulas(Punctuated<Formula, Token![;]>);

impl Parse for Formulas {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        <Punctuated<Formula, Token![;]>>::parse_terminated(input).map(Self)
    }
}

#[derive(Clone, Debug)]
struct Formula(Ident, Exprs);

/// Sequence: e1 e2
// Sequence(Vec<Expr>)
impl Parse for Formula {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(Parse::parse(input)?, {
            <Token![<]>::parse(input)?;
            <Token![-]>::parse(input)?;
            Parse::parse(input)?
        }))
    }
}

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

/// https://en.wikipedia.org/wiki/Parsing_expression_grammar#Composite_parsing_expressions
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
    ZeroOrMore(ExprToken),
    /// One-or-more: e+
    OneOrMore(ExprToken),
    /// Optional: e?
    Optional(ExprToken),
    /// And-predicate: &e
    AndPredicate(ExprToken),
    /// Not-predicate: !e
    NotPredicate(ExprToken),
    ExprToken(ExprToken),
}

impl Parse for ExprOther {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        (|| {
            let lookahead_input = input.fork();
            if let Ok(expr_token) = ExprToken::parse(&lookahead_input) {
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
            if let Ok(expr_token) = ExprToken::parse(&lookahead_input) {
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
            if let Ok(expr_token) = ExprToken::parse(&lookahead_input) {
                if lookahead_input.peek(Token![?]) {
                    input.advance_to(&lookahead_input);
                    input.parse::<Token![?]>().unwrap();
                    return Ok(Self::ZeroOrMore(expr_token));
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
        .or_else(|_| ExprToken::parse(input).map(Self::ExprToken))
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
            Parse::parse(&content).map(Self::Group)
        } else {
            Parse::parse(input)
                .map(Self::Ident)
                .or_else(|_| Parse::parse(input).map(Self::Literal))
        }
    }
}
