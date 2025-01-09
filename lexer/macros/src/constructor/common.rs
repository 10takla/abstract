use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use std::collections::HashMap;
use std::fmt::Debug;

pub fn common(
    errors: HashMap<Ident, Vec<Ident>>,
    v: [Vec<Ident>; 4],
) -> TokenStream2 {
    let construct = v.iter().flatten().collect::<Vec<_>>();

    if construct.is_empty() {
        return quote! {};
    }

    let [init_item, enum_name, items, cons_name] = &v;
    let construct_parse = [init_item, enum_name, cons_name]
        .into_iter()
        .flatten()
        .map(|tmp| {
            quote! {
                Construct::#tmp => {
                    #tmp::recog(arg, l).map(|v| ConstructItem::#tmp(v))
                }
            }
        })
        .chain(items.into_iter().map(|items| {
            quote! {
                Construct::#items => {
                    Ok(ConstructItem::#items(#items::recog(arg, l)))
                }
            }
        }));
    
       

    let errors =  [init_item, cons_name].into_iter().flatten()
    .map(|v| {
        if let Some(errs) = errors.get(v).cloned() && errs.clone().into_iter().next().is_some() {
            quote! {
                #[derive(Clone, Debug, PartialEq)]
                pub enum [<#v Error>] {
                    #(#errs),*
                }
            }
        } else {
            quote! {
                #[derive(Clone, Debug, PartialEq)]
                pub enum [<#v Error>] {
                    Some
                }
            }
        }
    });

    quote! {
        #[derive(Clone, Debug, Eq, PartialEq, Hash)]
        pub enum Construct {
            #(#construct),*
        }

        #[derive(Clone, Debug, PartialEq)]
        enum ConstructItem {
            #(#construct(#construct)),*
        }

        trait ConstructParse<const N: usize> {
            fn recog(&self, arg: &mut ParseArgs, l: usize) -> Result<[ConstructItem; N], Diag>;
        }
        impl<const N: usize> ConstructParse<N> for [Construct; N] {
            fn recog(&self, arg: &mut ParseArgs, l: usize) -> Result<[ConstructItem; N], Diag> {
                self.iter().map(|item| {
                    match item {
                        #(#construct_parse),*
                    }
                }).collect::<Result<Vec<_>, _>>().map(|v| v.try_into().unwrap())
            }
        }

        impl Slicable for ConstructItem {
            fn slice(&self) -> Slice {
                match self {
                    #(Self::#construct(v) => v.slice()),*
                }
            }
        }

        paste! {
            #[derive(Clone, Debug, PartialEq)]
            pub enum ErrorType {
                Reg(&'static str),
                LineOver,
                Any,
                #(#init_item([<#init_item Error>])),*,
                // #(#enum_name([<#enum_name Error>])),+,
                #(#cons_name([<#cons_name Error>])),*


                // Ident(IdentError),
                // String(StringError),
            }
            #(#errors)*
        }
    }
}
