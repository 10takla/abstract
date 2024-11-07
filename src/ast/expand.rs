use super::name_resolve::Refs;
use crate::lexer::items::{
    item::{
        block::{
            distruct::{named::CallBlockDistruct, Distruct},
            init::{named::NamedBlock, Init},
            Block,
        },
        Item,
    },
    Items,
};

pub fn expand(items: &Items, refs: &Refs) -> Items {
    let mut new_items = Vec::new();

    for item in items.clone().iter() {
        if let Item::Block(block) = item {
            match block {
                Block::Init(Init::Unnamed(unnamed_block)) => {
                    new_items.extend(expand(&unnamed_block.items, refs).0);
                    continue;
                }
                Block::Distruct(Distruct::Call(CallBlockDistruct { .. })) => {
                    if let Some(on) = refs.get(item) {
                        if let Item::Block(Block::Init(Init::Named(NamedBlock { block, .. }))) = on
                        {
                            let n = expand(&block.items, refs);
                            new_items.extend(n.0);
                            continue;
                        }
                    }
                    continue;
                }
                Block::Init(Init::Named(_)) => {
                    continue;
                }
                _ => {}
            }
        }

        new_items.push(item.clone());
    }

    Items(new_items)
}

mod tmp {}
