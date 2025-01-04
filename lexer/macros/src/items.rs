use crate::{check_pass_fail, fast_group, fast_ident, fast_puncts, COMMON};
use proc_macro2::{Group, Ident, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{parse2, LitStr};

pub fn items(g: &Group) -> (TokenStream2, Vec<Ident>) {
    let mut iter = g.stream().into_iter().peekable();
    let mut vec = vec![];
    let mut items_names = vec![];
    while let Some(item) = iter.next() {
        let items = fast_ident(&item).unwrap();

        items_names.push(items.clone());
        
        let item = {
            let Ok(v) = fast_group(&mut iter) else {
                continue;
            };

            fast_ident(&v.stream().into_iter().next().unwrap()).unwrap()
        };

        let break_ = fast_ident(&iter.next().unwrap()).unwrap();
        
        vec.push(
            quote! {
                #[derive(Debug, Clone, Deref)]
                pub struct #items(
                    Vec<#item>
                );

                impl Slicable for #items {
                    fn slice(&self) -> Slice {
                        self.first()
                            .map(|v| RangeInclusive::new(*v.slice().start(), *self.last().unwrap().slice().end()))
                            .unwrap()
                    }
                }

                impl #items {
                    pub fn recog(arg: &mut ParseArgs, l: usize) -> Self {
                        let mut vec = vec![];
                        loop {
                            if arg.code.cursor == arg.code.source.len() {
                                break;
                            }
                            let i = arg.code.cursor;
                            match #item::recog(arg, l) {
                                Ok(v) => {
                                    // не влияет на алгоритм, но очищает ненужную память, ускоряет поиск, в списке
                                    arg.c_a_d.borrow_mut().cache.pass.clear();
                                    arg.c_a_d.borrow_mut().cache.fails.clear();
                                    vec.push(v);
                                }
                                Err(e) => {
                                    if #break_::recog(&mut arg.clone(), l).is_ok() {
                                        break;
                                    } else {
                                        arg.code.cursor = *e.end();
                                        // println!("ERROR {e:?}");
                                        arg.c_a_d.borrow_mut().errors.push(e);
                                        continue;
                                    }
                                }
                            };
                        }
                        Self(vec)
                    }
                }           
            }
        );
    }
    (quote! {#(#vec)*}, items_names)
}
