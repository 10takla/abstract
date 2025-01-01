use crate::{check_pass_fail, fast_group, fast_ident, fast_puncts, COMMON};
use proc_macro::{Diagnostic, Level};
use proc_macro2::{Group, Ident, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::Index;

pub fn constructs(g: &Group, items: &Vec<Ident>) -> (TokenStream2, Vec<Ident>) {
    let mut iter = g.stream().into_iter().peekable();
    let mut vec = vec![];
    let mut construct_names = vec![];
    Diagnostic::new(Level::Warning, "fsdfsdf").emit();

    // panic!("dsfsf");
    while let Some(item) = iter.next() {
        let Ok(cons_name) = fast_ident(&item) else {
            break
        };
        let Ok(_) = fast_puncts("->", &mut iter) else {
            break
        };

        let mut cons_item = vec![];
        let mut tmp = vec![];
        while let Some((v, maybe)) = {
            {
                let mut iter = iter.clone();
                iter.next().and_then(|v| fast_ident(&v).ok())
                .and_then(|v| {
                    let maybe = fast_group(&mut iter).ok().map(|v| {
                        let ignore = fast_ident(&v.stream().into_iter().next().unwrap()).unwrap();
                        quote!{#ignore::recog(arg, l + 1);}
                    });
                    if let Some(vv) = iter.next() {
                        fast_ident(&vv).ok().map(|_| (v.clone(), maybe.clone()))
                    } else {
                        Some((v, maybe))
                    }
                })
            }.map(|v| {
                iter.next().unwrap();
                v.1.as_ref().map(|_| {
                    iter.next().unwrap();
                });
                v
            })
        }
        {
            cons_item.push(v.clone());
            tmp.push(
                if items.contains(&v) {
                    quote! {
                        let v = #v::recog(arg, l + 1);
                        cache_if_error.push((
                            Construct::#v,
                            when_not_fail,
                            ConstructItem::#v(v.clone()),
                        ));
                        #maybe
                        v
                    }
                } else {
                    quote! {
                        match #v::check_pass(arg, l + 1) {
                            Some((i, v)) => {
                                if let Some(v) = ptr {
                                    if v != i {
                                        ptr = None
                                    }
                                } else {
                                    ptr = Some(i);
                                }
            
                                arg.c_a_d.borrow_mut().cache.pass[i].index += 1;
        
                                #maybe
        
                                v
                            }
                            None => {
                                match #v::parse(arg, l + 1) {
                                    Ok(v) => {
                                        cache_if_error.push((
                                            Construct::#v,
                                            when_not_fail,
                                            ConstructItem::#v(v.clone()),
                                        ));
                                        #maybe
                                        v
                                    }
                                    Err(e) =>  {
                                        if !cache_if_error.is_empty() {
                                            if let Some(i) = ptr {
                                                let v = &mut arg.c_a_d.borrow_mut().cache.pass[i];
                                                v.index = 0;
                                                v.items.extend(cache_if_error.clone());
                                            } else {
                                                arg.c_a_d.borrow_mut().cache.pass.push(PassList::new(cache_if_error.clone()));
                                            }
                                        }
                                        arg.c_a_d.borrow_mut().cache.fails.insert((Construct::#v, when_not_fail), e.clone());
                                        return Err(e);
                                    }
                                }
                            }
                        }
                    }
                }
            );
        }
        if cons_item.len() == 0 {
            panic!("expect elements")
        }
        let n = Index::from(cons_item.len()-1);
        let common = &*COMMON;
        let check_pass_fail = check_pass_fail("cons", &cons_name);
        vec.push(
            quote! {
                #[derive(Clone, Debug)]
                pub struct #cons_name(#( pub #cons_item ),*);
                impl CommonTypes for #cons_name {
                    const CONST: Construct = Construct::#cons_name;
                }
                impl #cons_name {
                    #common

                    #check_pass_fail

                    // нет необходимости в consume ведь `items` сами это делаеют
                    fn parse(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                        print_colored(format!("cons {:?} {:?} {}", Self::CONST, arg.c_a_d.borrow().cache.pass, arg.code.cursor), l);
                        Self::after_debug(arg, l).map(|v| {
                            print_colored(pass_or_fail::<true>(), l);
                            v
                        }).map_err(|e| {
                            print_colored(pass_or_fail::<false>(), l);
                            e
                        })
                    }

                    fn after_debug(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                        let mut cache_if_error: Vec<(Construct, Pos, ConstructItem)> = Default::default();
                        let mut ptr = Default::default();
                        Ok(
                            Self(
                                #(
                                    {
                                        let when_not_fail = arg.code.cursor;
                                        #tmp
                                    }
                                ),*
                            )
                        )
                    }
                }

                impl Slicable for #cons_name {
                    fn slice(&self) -> Slice {
                        let start = self.0.slice();
                        let end = self.#n.slice();
                        *start.start()..=*end.end()
                    }
                }
            }
        );

        construct_names.push(cons_name);
    }
    (quote! {#(#vec)*}, construct_names)
}
