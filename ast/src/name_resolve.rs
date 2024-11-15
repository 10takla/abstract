use lexer::items::{
    item::{
        assign_expr::{assign::Assign, AssignExpr, AssignExprType},
        block::{
            distruct::{self, init::InitBlockDistruct, named::CallBlockDistruct, Distruct},
            init::{named::NamedBlock, Init},
            Block,
        },
        Item,
    },
    Items,
};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    AlreadyInited,
    NotInited,
}

pub type Refs<'a> = HashMap<&'a Item, &'a Item>;

pub fn name_resolve<'a>(
    items: &'a Items,
    globals: Option<HashMap<&str, &'a Item>>,
) -> (Refs<'a>, Vec<(Error, &'a Item)>) {
    let mut errors = vec![];

    let mut item_refs: Refs = HashMap::new();
    let (mut block_stack, mut assign_stack): (HashMap<&str, &Item>, HashMap<&str, &Item>) =
        (HashMap::new(), globals.map(|v| v).unwrap_or_default());

    for item in items.0.iter() {
        if let Item::Block(Block::Init(init)) = item {
            let (_, res) = match init {
                Init::Named(v) => name_resolve(&v.block.items, None),
                Init::Unnamed(v) => name_resolve(&v.items, Some(assign_stack.clone())),
            };
            errors.extend(res);
        }

        match item {
            Item::Block(block) => match block {
                Block::Distruct(distruct) => match distruct {
                    Distruct::Call(call) => {
                        item_refs.insert(item, {
                            if let Some(v) = block_stack.get(call.name.source) {
                                v
                            } else {
                                errors.push((Error::NotInited, &item));
                                continue;
                            }
                        });
                    }
                    Distruct::Init(init) => {
                        if block_stack.get(init.named_block.name.source).is_none() {
                            block_stack.insert(init.named_block.name.source, item);
                        } else {
                            errors.push((Error::AlreadyInited, &item));
                            continue;
                        }
                    }
                },
                Block::Init(Init::Named(named_block)) => {
                    if block_stack.get(named_block.name.source).is_none() {
                        block_stack.insert(named_block.name.source, item);
                    } else {
                        errors.push((Error::AlreadyInited, &item));
                        continue;
                    }
                }
                _ => {}
            },
            Item::AssignExpr(assign_expr) => match assign_expr {
                AssignExpr { type_, val } => match type_ {
                    AssignExprType::AssignAnd(_) => {
                        item_refs.insert(item, {
                            if let Some(v) = assign_stack.get(val.left.source) {
                                v
                            } else {
                                errors.push((Error::NotInited, &item));
                                continue;
                            }
                        });
                    }
                    AssignExprType::Assign => {
                        assign_stack.insert(val.left.source, item);
                    }
                },
            },
            Item::Ident(ident) => {
                item_refs.insert(item, {
                    if let Some(v) = assign_stack.get(ident.source) {
                        v
                    } else {
                        errors.push((Error::NotInited, &item));
                        continue;
                    }
                });
            }
            _ => {}
        }
    }

    (item_refs, errors)
}
