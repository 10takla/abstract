use super::name_resolve::Refs;
use lexer::items::{
    item::{
        block::{
            distruct::{init::InitBlockDistruct, call::CallBlockDistruct, BlockDistruct},
            init::{named::NamedBlock, InitBlock},
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
                    InitBlock::Unnamed(unnamed_block) => {
                        new_items.extend(expand(&unnamed_block.items, refs).0);
                        continue;
                    }
                    InitBlock::Named(_) => {
                        continue;
                    }
                },
                Block::Distruct(block) => match block {
                    BlockDistruct::Call(CallBlockDistruct { .. }) => {
                        if let Some(on) = refs.get(item) {
                            if let Item::Block(block) = on {
                                let mut fast = |NamedBlock { block, .. }: &NamedBlock<'s>| {
                                    let n = expand(&block.items, refs);
                                    new_items.extend(n.0);
                                };
                                match block {
                                    Block::Init(InitBlock::Named(named_block)) => {
                                        fast(named_block);
                                        continue;
                                    }
                                    Block::Distruct(BlockDistruct::Init(InitBlockDistruct {
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
                    BlockDistruct::Init(InitBlockDistruct { named_block, .. }) => {
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
