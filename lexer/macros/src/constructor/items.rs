use super::{check_pass_fail, fast_group, fast_ident, fast_ident2, fast_puncts, tmp, tmp3, COMMON};
use proc_macro2::{token_stream::IntoIter, Group, Ident, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::{
    iter::Peekable,
    panic::{catch_unwind, AssertUnwindSafe},
};
use syn::{parse2, LitStr};

pub fn items(mut iter: IntoIter) -> (TokenStream2, Vec<Ident>) {
    let (mut tokens, mut names): (TokenStream2, Vec<Ident>) = Default::default();

    while iter.clone().next().is_some() {
        let v = tmp3(&mut iter, items_recognize).unwrap();
        tokens.extend(v.0);
        names.push(v.1.clone());
    }

    (tokens, names)
}

pub fn items_recognize(iter: &mut Peekable<IntoIter>) -> ((TokenStream2, Ident), usize) {
    let mut counter = 0;

    let name = fast_ident2(iter).unwrap();
    counter += 1;

    let _ = fast_puncts("!", iter).map(|v| {
        counter += 1;
    });

    let item = fast_ident2(&mut fast_group(iter).unwrap().stream().into_iter().peekable()).unwrap();
    counter += 1;

    let join = fast_group(iter)
        .and_then(|v| fast_ident2(&mut v.stream().into_iter().peekable()))
        .map(|v| {
            counter += 1;
            quote! {
                #v::recog(arg, l);
            }
        })
        .ok();

    let break_ = fast_ident2(iter).unwrap();
    counter += 1;

    let tokens = quote! {
        #[derive(Debug, Clone, Deref)]
        pub struct #name(
            Vec<#item>
        );

        impl Slicable for #name {
            fn slice(&self) -> Slice {
                self.first()
                    .map(|v| RangeInclusive::new(*v.slice().start(), *self.last().unwrap().slice().end()))
                    .unwrap()
            }
        }

        impl #name {
            pub fn recog(arg: &mut ParseArgs, l: usize) -> Self {
                let mut vec = vec![];
                loop {
                    if arg.code.cursor >= arg.code.source.len() {
                        break;
                    }
                    match #item::recog(arg, l) {
                        Ok(v) => {
                            // не влияет на алгоритм, но очищает ненужную память, ускоряет поиск в списке
                            arg.c_a_d.borrow_mut().cache.pass.clear();
                            vec.push(v);
                            #join
                        }
                        Err(e) => {
                            if #break_::recog(&mut arg.clone(), l).is_ok() {
                                break;
                            } else {
                                let e = arg.c_a_d.borrow().clone().cache.check(e);
                                arg.code.cursor = *e.end() + 1;
                                arg.c_a_d.borrow_mut().errors.push(e);

                                #join
                                continue;
                            }
                        }
                    };
                }
                Self(vec)
            }
        }
    };

    ((tokens, name), counter)
}
