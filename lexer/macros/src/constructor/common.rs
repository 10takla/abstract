use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use std::collections::HashMap;

pub fn common(errors: HashMap<Ident, Vec<Ident>>, all_items: [Vec<Ident>; 4]) -> TokenStream2 {
   
    let v = all_items.iter().flatten().collect::<Vec<_>>();
    
    if v.is_empty() {
        return quote! {};
    }

    let [init_item, enum_name, items, cons_name] = &all_items;
    let construct_parse = [init_item, enum_name, cons_name]
        .into_iter()
        .flatten()
        .map(|tmp| {
            quote! {
                Construct::#tmp => {
                    #tmp::recog(arg, l).map(ConstructItem::#tmp)
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

    let errors = [init_item, cons_name, items]
        .into_iter()
        .flatten()
        .map(|v| {
            if let Some(errs) = errors.get(v).cloned()
                && errs.clone().into_iter().next().is_some()
            {
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

    let v2 = init_item.into_iter().map(|init_item| {
            quote! {#init_item([<#init_item Error>])}
        })
        .chain(
            enum_name.into_iter().map(|enum_name| {
                quote! {#enum_name(Vec<Diag>)}
            })
        )
        .chain(
            cons_name.into_iter().map(|cons_name| {
                quote! {#cons_name([<#cons_name Error>])}
            })
        );

    quote! {
        #[derive(Clone, Debug, Eq, PartialEq, Hash)]
        pub enum Construct {
            #(#v),*
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum ConstructItem {
            #(#v(#v)),*
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
                    #(Self::#v(v) => v.slice()),*
                }
            }
        }

        paste! {
            #[derive(Clone, Debug, PartialEq)]
            pub enum ErrorType {
                Reg(&'static str),
                LineOver,
                Any,
                #(#v2),*
            }
            #(#errors)*
        }
    }
}
