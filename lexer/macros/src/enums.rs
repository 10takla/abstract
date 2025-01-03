use crate::{check_pass_fail, fast_ident, fast_puncts, COMMON};
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

                    fn parse(arg: &mut ParseArgs) -> <Self as CommonTypes>::Output {
                        arg.print.print_colored(format!("enum {:?} {:?} {}", Self::CONST, arg.c_a_d.borrow().cache.pass, arg.code.cursor));
                        Self::consume_parse(arg).map(|v| {
                            arg.print.pass_or_fail::<true>();
                            v
                        }).map_err(|e| {
                            arg.print.pass_or_fail::<false>();
                            e
                        })
                    }

                    // есть необходимость в `consume` ведь мы делаем `arg.clone`
                    fn consume_parse(arg: &mut ParseArgs) -> <Self as CommonTypes>::Output {
                        Self::after_debug(arg)
                        .map(|v| {
                            arg.code.cursor = v.slice().end() + 1;
                            v
                        })
                    }

                    // enum не кешируется, потому что:
                    // 1. состоит из токенов и конструкций, которые кешируются
                    // 2. enum состоит из вариций, если кешировать одну это значит кешировать любую другую
                    // fn cache_parse(arg: &mut ParseArgs) -> Self::Output
                    fn after_debug(arg: &ParseArgs) -> <Self as CommonTypes>::Output {
                        let mut error: Option<Diag> = None;

                        match #first::recog(arg.clone().add_level()).map(Self::#first) {
                            Ok(v) => return Ok(v),
                            Err(e) =>  {
                                match error {
                                    Some(v) if e.end() > v.end() => error = Some(e.clone()),
                                    None => error = Some(e.clone()),
                                    _ => {}
                                };
                                #(
                                    match #other::recog(arg.clone().add_level()).map(Self::#other) {
                                        Ok(v) => return Ok(v),
                                        Err(e) =>  {
                                            match error {
                                                Some(v) if e.end() > v.end() => error = Some(e.clone()),
                                                None => error = Some(e.clone()),
                                                _ => {}
                                            };
                                        }
                                    }
                                )*
                            }
                        }

                        Err(error.clone().unwrap())
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
