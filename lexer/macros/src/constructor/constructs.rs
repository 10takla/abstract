use std::{iter::Peekable, panic::{catch_unwind, AssertUnwindSafe}};
use super::{check_pass_fail, fast_group, fast_ident, fast_ident2, fast_puncts, tmp,  tmp5, COMMON};
use proc_macro2::{token_stream::IntoIter, Group, Ident, Literal, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse::Peek, Index};


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

fn head(iter: &mut Peekable<IntoIter>, counter: &mut usize) -> Result<(Ident, Option<Ident>), String> {
    let cons_name = fast_ident2(iter)?;
    *counter += 1;
    let common_ignore = fast_group(iter).ok().map(|v| {
        *counter += 1;
        fast_ident2(&mut v.stream().into_iter().peekable()).unwrap()
    });
    fast_puncts("->", iter)?;
    *counter += 2;
    Ok((cons_name, common_ignore))
}

pub fn constructs_reckog(
    iter: &mut IntoIter, items: &Vec<Ident>
) -> ((TokenStream2, Vec<Ident>), Result<(), String>) {
    let mut vec = vec![];
    let mut construct_names = vec![];
    
    let res = catch_unwind(AssertUnwindSafe(|| {
        let mut iter = iter.clone().peekable();
        while iter.peek().is_some() {
            let Ok((cons_name, common_ignore)) = head(&mut iter, &mut 0) else {
                break
            };
    
            let (mut cons_item, mut tmp) = (Vec::default(), Vec::default());
            while let Some((v, maybe)) = {
                head(&mut iter.clone(), &mut 0).err()
                    .and_then(|_| fast_ident2(&mut iter).ok())
                    .map(|v| (
                        v,
                        common_ignore.clone()
                            .or_else(|| {
                                fast_group(&mut iter).ok().map(|v| {
                                    fast_ident(&v.stream().into_iter().next().expect("1")).expect("2")
                                })
                            })
                            .map(|v| {
                                quote! {#v::recog(arg, l + 1);}
                            })
                    ))
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
                    #[derive(Clone, Debug, PartialEq)]
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

pub fn construct_recognize(
    iter: &mut Peekable<IntoIter>
) -> ((Ident, Vec<(Ident, bool, Option<Ident>)>), usize) {
    let mut counter = 0;

    let (name, common_ignore) = head(iter, &mut counter).unwrap();

    let mut items_i = vec![];
    while let Some(v) = {
        {
            let mut iter2 = iter.clone();
            fast_ident2(&mut iter2).ok()
                .and_then(|v| {
                    let is_box = fast_puncts("!", &mut iter2).is_ok();
                    let maybe = fast_group(&mut iter2).ok().map(|v| {
                        fast_ident(&v.stream().into_iter().next().unwrap()).unwrap()
                    });
                
                    if let Some(vv) = iter2.next() {
                        fast_ident(&vv).ok()
                            .map(|_| (v.clone(), is_box, maybe.clone()))
                    } else {
                        Some((v, is_box, maybe))
                    }
            })
        }
        .map(|v| {
            counter += 1;
            iter.next().unwrap();
            if v.1 {
                counter += 1;
                iter.next().unwrap();
            }
            (
                v.0,
                v.1,
                common_ignore.clone().or_else(|| {
                    v.2.as_ref().map(|v| {
                        counter += 1;
                        iter.next().unwrap();
                    });
                    v.2
                })
            )
        })
    } {
        items_i.push(v);
    }
    common_ignore.and_then(|_| items_i.last_mut())
        .map(|v| {
            v.2 = None;
        });

    if items_i.len() == 0 {
        panic!("construct expect elements")
    }
    
    ((name, items_i), counter)
}

pub fn construct_tokens(
    name: &Ident,
    items: &Vec<Ident>,
    items_i: Vec<(Ident, bool, Option<Ident>)>
) -> TokenStream2 {
    let tmp = items_i.iter().map(|(v, is_box, maybe)| {
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
            let is_box = is_box.then(|| quote! {
                .map(|v| Box::new(v))
            });
            let maybe = maybe.as_ref().map(|v| quote! {
                .map(|v| {
                    #v
                    v
                })
            });
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
                    #is_box
                    #maybe?
            }
        }
    }).collect::<Vec<_>>();

    let cons_item = items_i.iter().map(|(name, is_box, _)| {
        if *is_box {
            quote! {Box<#name>}
        } else {
            quote! {#name}
        }
    });

    let n = Index::from(items_i.len() - 1);
    let common = &*COMMON;

    quote! {
        #[derive(Clone, Debug, PartialEq)]
        pub struct #name(#( pub #cons_item ),*);
        impl CommonTypes for #name {
            const CONST: Construct = Construct::#name;
        }

        impl Recog for #name {
            fn parse2(arg: &mut ParseArgs, l: usize) -> <Self as CommonTypes>::Output {
                ConstructRecog::parse(arg, l)
            }
        }

        impl CacheCheck for #name {
            const PREFIX: &str = "cons";

            fn unwrap_item(item: ConstructItem) -> Self {
                let ConstructItem::#name(v) = item else {unreachable!()};
                v
            }
        }

        impl ConstructRecog for #name {
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