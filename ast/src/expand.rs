use super::name_resolve::Refs;
use lexer::items::{
    item::{
        block::{
            distruct::{init::InitBlockDistruct, named::CallBlockDistruct, Distruct},
            init::{named::NamedBlock, Init},
            Block,
        },
        Item,
    },
    Items,
};

pub fn expand<'s, 'i>(items: &'i Items<'s>, refs: &Refs<'s, 'i>) -> Items<'s> {
    let mut new_items: Vec<Item<'s>> = Vec::new();

    for item in items.iter() {
        if let Item::Block(block) = item {
            match block {
                Block::Init(tmp) => match tmp {
                    Init::Unnamed(unnamed_block) => {
                        new_items.extend(expand(&unnamed_block.items, refs).0);
                        continue;
                    }
                    Init::Named(_) => {
                        continue;
                    }
                },
                Block::Distruct(block) => match block {
                    Distruct::Call(CallBlockDistruct { .. }) => {
                        if let Some(on) = refs.get(item) {
                            if let Item::Block(block) = on {
                                let mut fast = |NamedBlock { block, .. }: &NamedBlock<'s>| {
                                    let n = expand(&block.items, refs);
                                    new_items.extend(n.0);
                                };
                                match block {
                                    Block::Init(Init::Named(named_block)) => {
                                        fast(named_block);
                                        continue;
                                    }
                                    Block::Distruct(Distruct::Init(InitBlockDistruct {
                                        named_block,
                                        ..
                                    })) => {
                                        fast(named_block);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        continue;
                    }
                    Distruct::Init(InitBlockDistruct { named_block, .. }) => {
                        let n = expand(&named_block.block.items, refs);
                        new_items.extend(n.0);
                        continue;
                    }
                },
            }
        }

        new_items.push(item.clone());
    }

    Items(new_items)
}

mod tmp {}
