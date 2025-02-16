use super::{fast_group, fast_ident, fast_ident2, tmp, tmp3, tmp5};
use proc_macro2::{token_stream::IntoIter, Group, Ident, Literal, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    iter::Peekable,
    panic::{catch_unwind, AssertUnwindSafe},
};
use syn::{parse2, LitStr};

pub fn tokens(mut iter: IntoIter) -> (TokenStream2, Vec<Ident>, HashMap<Ident, Vec<Ident>>) {
    let (mut tokens, mut names, mut errors): (
        TokenStream2,
        Vec<Ident>,
        HashMap<Ident, Vec<Ident>>,
    ) = Default::default();

    while iter.clone().next().is_some() {
        let v = tmp3(&mut iter, token_recognize).unwrap();
        tokens.extend(v.0);
        names.push(v.1.clone());
        v.2.map(|t| errors.insert(v.1, t));
    }
    
    (tokens, names, errors)
}

pub fn token_recognize(
    iter: &mut Peekable<IntoIter>,
) -> ((TokenStream2, Ident, Option<Vec<Ident>>), usize) {
    let mut counter = 0;

    let name = fast_ident2(iter).unwrap();
    counter += 1;

    let mut errors = None;

    let body = match iter.next().unwrap() {
        TokenTree::Literal(v) => {
            parse2::<LitStr>(TokenTree::from(v.clone()).into()).unwrap();
            counter += 1;
            quote! {
                reg_observe(arg, #v).map_err(|v| (v..=v, ErrorType::Reg(#v)))
            }
        }
        TokenTree::Group(body) => {
            let e = fast_group(iter).unwrap();
            counter += 1;
            errors = Some(
                body.stream()
                    .into_iter()
                    .map(|v| fast_ident(&v).unwrap())
                    .collect(),
            );
            counter += 1;
            quote! {
                (|arg: &ParseArgs| #e)(arg)
            }
        }
        _ => unreachable!(),
    };

    let marker = Ident::new(&format!("{name}Marker"), Span::call_site());

    let tokens = quote! {
        #[derive(Clone, Debug, PartialEq)]
        pub struct #marker;

        pub type #name = Token<#marker>;

        impl #name {
            pub const fn new(slice: Slice) -> Self {
                Self(#marker, slice)
            }
        }

        impl ConstructMarker for #name {
            fn item(&self, arg: &mut ParseArgs, l: usize) -> Result<ConstructItem, Diag> {
                #name::recog(arg, l).map(ConstructItem::#name)
            }
        }

        impl ConstructTypes for #name {
            const CONST: Construct = Construct::#name;
        }

        impl CommonTypes for #name {}

        impl Recog for #name {
            fn parse2(arg: &mut ParseArgs, l: usize) -> Self::Output {
                <#name as TokenRecog<#marker>>::parse(arg, l)
            }
        }

        impl CacheCheck for #name {
            const PREFIX: &str = "token";

            fn unwrap_item(item: ConstructItem) -> Self {
                let ConstructItem::#name(v) = item else {unreachable!()};
                v
            }
        }

            impl TokenRecog<#marker> for #name {
            fn after_debug(arg: &ParseArgs) -> Self::Output {
                #body.map(|v| Self(#marker, v))
                    .map_err(|(slice, error)| Diag {
                        slice,
                        source: arg.code.source.clone(),
                        error,
                        type_: Self::CONST
                    })
            }
        }
    };

    ((tokens, name, errors), counter)
}
