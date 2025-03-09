use std::{collections::HashSet, default};
use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, Literal};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseBuffer, ParseStream}, parse_macro_input, punctuated::{IntoIter, Iter, Punctuated}, token::{Brace, Bracket, Break, Paren}, AngleBracketedGenericArguments, Expr, Fields, FieldsUnnamed, Ident, ItemEnum, ItemStruct, Lit, Meta, MetaList, PathArguments, Token, Variant
};

pub fn constructor(input: TokenStream) -> TokenStream {
    let items: Items = parse_macro_input!(input);

    #[derive(Default)]
    struct Tmp {
        enums: HashSet<Ident>,
        tokens: HashSet<Ident>,
        seqences: HashSet<Ident>,
        items: HashSet<Ident>,
    }
    
    let mut names = Tmp::default();
    for Item(name, type_) in &items.0 {
        match type_ {
            ItemType::Enum(..) => {names.enums.insert((*name).clone());},
            ItemType::Items(..) => {names.items.insert((*name).clone());},
            ItemType::Token(..) => {names.tokens.insert((*name).clone());},
            ItemType::Seq(..) => {names.seqences.insert((*name).clone());},
        }
    }

    let get_type = |v| {
        if names.seqences.contains(&v) {
            quote! {<Seq<#v>>}
        } else {
            quote! {#v}
        }
    };

    let mut ts = proc_macro2::TokenStream::new();
    for Item(name, type_) in items.0 {
        ts.extend(
            match type_ {
                ItemType::Enum(EnumInput(a)) => {
                    let n = a.len();
    
                    let a = a.into_iter();
                    let b = a
                        .clone()
                        .into_iter()
                        .map(|v| (*v.name()).clone())
                        .collect::<Vec<_>>();

                    let c = a.map(|type_| {
                        match type_ {
                            Type::Ident(v) => get_type(v),
                            Type::WrapType(v, b) => quote! {#v::#b},
                        }
                    });
                    
                    // let impl_ = impl_enum(name, c,  n);

                    quote! {
                        #[derive(Spanable, Debug, PartialEq)]
                        pub enum #name {
                            #(#b(<#b as CommonRecog>::Output)),*
                        }
                        
                        impl EnumRecog for #name {
                            type Output = Self;
                            fn structure_assembling<'a>(
                                ctxt: &'a Ctxt,
                            ) -> Vec<Box<dyn core::ops::Fn() -> Result<Self::Output, CommonError> + 'a>> {
                                vec![
                                    #(
                                        Box::new(|| #c::recog(ctxt).map(Self::#b))
                                    ),*
                                ]
                            }
                        }
                    }
                },
                ItemType::Items(ItemsInput {item, break_, join, break_on_error}) => {
                    let v = if break_on_error {
                        quote! {ErrorBreakRepetition<#item>}
                    } else {
                        if let Some(v) = break_ {
                            quote! {BreakRepetition<#item, #v>}
                        }else {
                            quote! {Vec<#item>}
                        }
                    };
                    quote! {
                        type #name = #v;
                    }
                },
                ItemType::Token(TokenInput(reg)) => {
                    quote! {
                        paste!{
                            #[derive(Debug, PartialEq)]
                            #[allow(non_camel_case_types)]
                            pub struct [<#name Marker>];
                            
                            impl RegularToken for [<#name Marker>] {
                                const REG_EXPR: &'static str = #reg;
                            }
                            
                            pub type #name = Token<[<#name Marker>]>;
                        }
                    }
                },
                ItemType::Seq(SeqInput {items, join}) => {
                    let item_names = 
                        {
                            if let Some(v1) = join {
                                items.iter().enumerate()
                                    .map(|(i, v)| {
                                        if i == items.len() - 1 {
                                            vec![quote!{#v}]
                                        } else {
                                            vec![quote!{#v}, quote!{#v1}]
                                        }
                                    })
                                    .flatten()
                                    .collect::<Vec<_>>()
                            } else {
                                items.iter().map(|v| quote!{#v}).collect::<Vec<_>>()
                            }
                        };
                    quote! {
                        pub type #name = ( #(#item_names),* );
                    }
                },
            }
        );
    }

    ts.into()
}

#[derive(Clone, Debug)]
enum Type {
    Ident(Ident),
    WrapType(Ident, Ident),
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Type::Ident(a) => {
                a.to_tokens(tokens);
            }
            Type::WrapType(a, b) => {
                tokens.extend(
                    quote!{#a<#b>}
                );
            }
        }
    }
}

impl Parse for Type {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        Ok(if <Token![<]>::parse(&input).is_ok() {
            let v = input.parse()?;
            <Token![>]>::parse(&input)?;
            Type::WrapType(ident, v)
        } else {
            Type::Ident(ident)
        })
    }
}

struct Items(Vec<Item>);

impl Parse for Items {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut vec = vec![];
        loop {
            if let Ok(v) = Item::parse(input) {
                vec.push(v);
            } else {
                break;
            }
        }
        Ok(Self(vec))
    }
}
struct Item(Ident, ItemType);

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(Parse::parse(input)?, Parse::parse(input)?))
    }
}

enum ItemType {
    Items(ItemsInput),
    Token(TokenInput),
    Seq(SeqInput),
    Enum(EnumInput),
}

impl Parse for ItemType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Parse::parse(input)
            .map(Self::Items)
            .or_else(|_| Parse::parse(input).map(Self::Token))
            .or_else(|_| Parse::parse(input).map(Self::Seq))
            .or_else(|_| Parse::parse(input).map(Self::Enum))
    }
}

struct EnumInput(Punctuated<Type, Token![|]>);

impl Parse for EnumInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::braced!(content in input);
        Punctuated::parse_terminated(&content).map(Self)
    }
}

struct SeqInput{
    items: Vec<Ident>, 
    join: Option<Ident>
}
impl Parse for SeqInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);

        let mut vec = vec![];
        loop {
            if let Ok(v) = Parse::parse(&content) {
                vec.push(v);
            } else {
                break;
            }
        }

        (!vec.is_empty())
            .then_some(Self{
                items: vec,
                join: {
                    let mut join = None;
                    if input.peek(Brace) {
                        let content;
                        syn::braced!(content in input);
                        for _ in 0..1 {
                            if let Ok(v) = Ident::parse(&content) && v == "join" {
                                join = Parse::parse(&content)?;
                            }
                        }
                    }
                    join
                }
            })
            .ok_or(input.error("No separator expected"))
    }
}
struct ItemsInput {
    item: Ident,
    break_: Option<Ident>,
    join: Option<Ident>,
    break_on_error: bool
}

impl Parse for ItemsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item = {
            <Token![<]>::parse(&input)?;
            let v = Parse::parse(&input)?;
            <Token![>]>::parse(&input)?;
            v
        };
        
        let mut break_ = None;
        let mut join = None;
        let mut break_on_error = false;

        if input.peek(Brace) {
            let content;
            syn::braced!(content in input);
    
            for _ in 0..3 {
                if Break::parse(&content).is_ok() {
                    break_ = Parse::parse(&content)?;
                } else if let Ok(v) = Ident::parse(&content)  {
                    if v == "join" {
                        join = Parse::parse(&content)?;
                    }
                    if v == "break_on_error" {
                        break_on_error = true;    
                    }
                }
            }
        }

        Ok(Self { item, break_, join, break_on_error })
    }
}

struct TokenInput(Literal);

impl Parse for TokenInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Parse::parse(input).map(Self)
    }
}




impl Type {
    fn name(&self) -> &Ident {
        match self {
            Type::Ident(v) => v,
            Type::WrapType(_, v) => v,
        }
    }
}