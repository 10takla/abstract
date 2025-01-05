use super::{check_pass_fail, fast_ident, fast_puncts, COMMON};
use proc_macro2::{Group, Ident, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{parse2, LitStr};

pub fn enums(g: &Group) -> (TokenStream2, Vec<Ident>) {
    let mut iter = g.stream().into_iter().peekable();
    let mut vec = vec![];
    let mut enum_names = vec![];

    while let Some(item) = iter.next() {
        let name = fast_ident(&item).unwrap();

        fast_puncts("->", &mut iter).unwrap();

        let mut item = vec![];
        while let Some(v) = iter.next() {
            let v = fast_ident(&v).unwrap();
            item.push(v);
            let Ok(_) = fast_puncts("|", &mut iter) else {
                break;
            };
        }

        let common = &*COMMON;
        let check_pass_fail = check_pass_fail("enum", &name);
        let (first, other) = item.split_first().unwrap();
        vec.push(
            quote! {
                #[derive(Clone, Debug)]
                pub enum #name {
                    #( #item(#item) ),*
                }

                impl CommonTypes for #name {
                    const CONST: Construct = Construct::#name;
                }
                impl #name {
                    #common

                    #check_pass_fail

                    fn parse(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                        arg.print.print_colored(arg.get_head("enum", Self::CONST, arg.code.cursor), l);
                        Self::consume_parse(arg, l).map(|v| {
                            arg.print.pass_or_fail::<true>(l);
                            v
                        }).map_err(|e| {
                            arg.print.pass_or_fail::<false>(l);
                            e
                        })
                    }

                    // есть необходимость в `consume` ведь мы делаем `arg.clone`
                    fn consume_parse(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                        Self::after_debug(arg, l)
                        .map(|v| {
                            arg.code.cursor = v.slice().end() + 1;
                            v
                        })
                    }

                    // enum не кешируется, потому что:
                    // 1. состоит из токенов и конструкций, которые кешируются
                    // 2. enum состоит из вариций, если кешировать одну это значит кешировать любую другую
                    // fn cache_parse(arg: &mut ParseArgs) -> Self::Output
                    fn after_debug(arg: &ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                        #first::recog(&mut arg.clone(), l + 1).map(Self::#first)
                        #(
                            .or_else(|error| {
                                #other::recog(&mut arg.clone(), l + 1).map(Self::#other)
                                .map_err(|e| {
                                    (e.end() > error.end()).then_some(e).unwrap_or(error)
                                })
                            })
                        )*
                    }
                }

                impl Slicable for #name {
                    fn slice(&self) -> Slice {
                        match self {
                            #( Self::#item(v) => v.slice() ),*
                        }
                    }
                }
            }
        );
        enum_names.push(name);
    }

    (quote! {#(#vec)*}, enum_names)
}
