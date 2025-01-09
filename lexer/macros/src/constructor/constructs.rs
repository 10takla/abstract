use std::{iter::Peekable, panic::{catch_unwind, AssertUnwindSafe}};
use super::{check_pass_fail, fast_group, fast_ident, fast_ident2, fast_puncts, tmp, tmp5, COMMON};
use proc_macro2::{token_stream::IntoIter, Group, Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::Index;


pub fn constructs(
    mut iter: IntoIter,  items: &Vec<Ident>
) -> (
    TokenStream2,
    Vec<Ident>,
) {
    let (v, res) = constructs_reckog(&mut iter, items);
    res.unwrap();
    v
}

pub fn constructs_reckog(
    iter: &mut IntoIter, items: &Vec<Ident>
) -> (
    (
        TokenStream2,
        Vec<Ident>,
    ),
    Result<(), String>,
) {
    let mut vec = vec![];
    let mut construct_names = vec![];
    
    let res = catch_unwind(AssertUnwindSafe(|| {
        let mut iter = iter.clone().peekable();
        while let Some(item) = iter.next() {
            let Ok(cons_name) = fast_ident(&item) else {
                break;
            };
            let Ok(_) = fast_puncts("->", &mut iter) else {
                break;
            };
    
            let mut cons_item = vec![];
            let mut tmp = vec![];
            while let Some((v, maybe)) = {
                {
                    let mut iter = iter.clone();
                    iter.next().and_then(|v| fast_ident(&v).ok()).and_then(|v| {
                        let maybe = fast_group(&mut iter).ok().map(|v| {
                            let ignore = fast_ident(&v.stream().into_iter().next().unwrap()).unwrap();
                            quote! {#ignore::recog(arg, l + 1);}
                        });
                        if let Some(vv) = iter.next() {
                            fast_ident(&vv).ok().map(|_| (v.clone(), maybe.clone()))
                        } else {
                            Some((v, maybe))
                        }
                    })
                }
                .map(|v| {
                    iter.next().unwrap();
                    v.1.as_ref().map(|_| iter.next().unwrap());
                    v
                })
            } {
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
                            #v::check_pass(arg, l + 1)
                                .map(|(i, v)| {
                                    if let Some(v) = ptr {
                                        if v != i {
                                            ptr = None
                                        }
                                    } else {
                                        ptr = Some(i);
                                    }
                                    arg.c_a_d.borrow_mut().cache.pass[i].index += 1;
                                    v
                                }).ok_or(())
                                .or_else(|_| {
                                    #v::parse(arg, l + 1)
                                        .map(|v| {
                                            cache_if_error.push((
                                                Construct::#v,
                                                when_not_fail,
                                                ConstructItem::#v(v.clone()),
                                            ));
                                            v
                                        })
                                        .map_err(|e| {
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
                                            e
                                        })
                                })
                                .map(|v| {
                                    #maybe
                                    v
                                })?
                        }
                    }
                );
            }
            if cons_item.len() == 0 {
                panic!("construct expect elements")
            }
            let n = Index::from(cons_item.len() - 1);
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
                            arg.print.print_colored(arg.get_head("cons", Self::CONST, arg.code.cursor), l);
                            Self::after_debug(arg, l).map(|v| {
                                arg.print.pass_or_fail::<true>(l);
                                v
                            }).map_err(|e| {
                                arg.print.pass_or_fail::<false>(l);
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
    }));

    for _ in &construct_names {
        iter.next().unwrap();
    }
    (
        (quote! {#(#vec)*}, construct_names),
        tmp(res),
    )
}

// pub fn construct_recognize(
//     iter: &mut IntoIter, items: &Vec<Ident>
// ) -> Result<(TokenStream2, Ident), String> {
//     catch_unwind(AssertUnwindSafe(|| construct_recogniz(&mut iter.clone().peekable(), items)))
//         .map(|(v, count)| {
//             for _ in 0..count {
//                 iter.next().unwrap();
//             }
//             v
//         })
//         .map_err(tmp5)
// }

pub fn construct_recognize(
    iter: &mut Peekable<IntoIter>
) -> ((Ident, Vec<(Ident, Option<Ident>)>), usize) {
    let mut counter = 0;

    let name = fast_ident2(iter).unwrap();
    counter += 1;

    fast_puncts("->", iter).unwrap();
    counter += 2;

    let mut items_i = vec![];
    while let Some((v, maybe)) = {
        {
            let mut iter = iter.clone();
            iter.next().and_then(|v| fast_ident(&v).ok()).and_then(|v| {
                let maybe = fast_group(&mut iter).ok().map(|v| {
                    let ignore = fast_ident(&v.stream().into_iter().next().unwrap()).unwrap();
                    counter += 1;
                    ignore
                });
               
                if let Some(vv) = iter.next() {
                    // panic!("{vv:?}");
                    fast_ident(&vv).ok()
                    // .and_then(|_| {
                    //     fast_puncts("(",  &mut iter).is_err().then_some(())
                    // })
                    .map(|_| (v.clone(), maybe.clone()))
                } else {
                    Some((v, maybe))
                }
            })
        }
        .map(|v| {
            counter += 1;
            iter.next().unwrap();
            v.1.as_ref().map(|_| iter.next().unwrap());
            v
        })
    } {
        items_i.push((v.clone(), maybe));
    }
    if items_i.len() == 0 {
        panic!("construct expect elements")
    }
    
    ((name, items_i), counter)
}

pub fn construct_tokens(
    name: &Ident,
    items: &Vec<Ident>,
    items_i: Vec<(Ident, Option<Ident>)>
) -> TokenStream2 {
    let tmp = items_i.iter().map(|(v, maybe)| {
        let maybe = maybe.as_ref().map(|v| quote! {#v::recog(arg, l + 1);});
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
                #v::check_pass(arg, l + 1)
                    .map(|(i, v)| {
                        if let Some(v) = ptr {
                            if v != i {
                                ptr = None
                            }
                        } else {
                            ptr = Some(i);
                        }
                        arg.c_a_d.borrow_mut().cache.pass[i].index += 1;
                        v
                    }).ok_or(())
                    .or_else(|_| {
                        #v::parse(arg, l + 1)
                            .map(|v| {
                                cache_if_error.push((
                                    Construct::#v,
                                    when_not_fail,
                                    ConstructItem::#v(v.clone()),
                                ));
                                v
                            })
                            .map_err(|e| {
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
                                e
                            })
                    })
                    .map(|v| {
                        #maybe
                        v
                    })?
            }
        }
    }).collect::<Vec<_>>();

    let cons_item = items_i.iter().map(|v| &v.0);

    let n = Index::from(items_i.len() - 1);
    let common = &*COMMON;
    let check_pass_fail = check_pass_fail("cons", &name);
     
    quote! {
        #[derive(Clone, Debug)]
        pub struct #name(#( pub #cons_item ),*);
        impl CommonTypes for #name {
            const CONST: Construct = Construct::#name;
        }
        impl #name {
            #common

            #check_pass_fail

            // нет необходимости в consume ведь `items` сами это делаеют
            fn parse(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                arg.print.print_colored(arg.get_head("cons", Self::CONST, arg.code.cursor), l);
                Self::after_debug(arg, l).map(|v| {
                    arg.print.pass_or_fail::<true>(l);
                    v
                }).map_err(|e| {
                    arg.print.pass_or_fail::<false>(l);
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

        impl Slicable for #name {
            fn slice(&self) -> Slice {
                let start = self.0.slice();
                let end = self.#n.slice();
                *start.start()..=*end.end()
            }
        }
    }
}