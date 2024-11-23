use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use super::{DiagParse, Diags};

#[derive(Default)]
pub struct Recognized {
    cache: HashMap<u64, Box<dyn AnyParse>>,
}

impl Recognized {
    pub fn get<T: AnyParse + 'static + Clone>(&self) -> Option<T> {
        self.cache
            .get(&Self::make_key::<T>())
            .and_then(|boxed| boxed.as_any().downcast_ref::<T>().cloned())
    }

    pub fn set<T: AnyParse + 'static>(&mut self, value: T) {
        let key = Self::make_key::<T>();
        self.cache.insert(key, Box::new(value));
    }

    pub fn make_key<T: 'static>() -> u64 {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<T>().hash(&mut hasher);
        hasher.finish()
    }
}

trait AnyParse {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Parse + 'static> AnyParse for Result<T, T::Diag> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

trait Parse: Sized {
    type Diag;
    fn parse(recognized: &mut Recognized) -> Result<Self, Self::Diag>;
}
trait SelectionParse: Parse {
    fn rec() -> Result<Self, Self::Diag> {
        Self::parse(&mut Recognized::default())
    }
}
trait RecognizeParse: Parse {
    fn rec(recognized: &mut Recognized) -> Result<Self, Self::Diag>
    where
        Result<Self, Self::Diag>: Clone + 'static,
    {
        recognized.get().unwrap_or_else(|| {
            let res = Self::parse(recognized);
            recognized.set(res.clone());
            res
        })
    }
}

enum Item {
    NamedBlock(NamedBlock),
    Ident(Ident),
}
enum ItemDiag {
    NamedBlock(NamedBlockDiag),
    Ident(IdentDiag),
}

impl SelectionParse for Item {}
impl Parse for Item {
    type Diag = ItemDiag;
    fn parse(recognized: &mut Recognized) -> Result<Self, Self::Diag> {
        NamedBlock::rec(recognized)
            .map(Self::NamedBlock)
            .map_err(ItemDiag::NamedBlock)
            .or_else(|_| {
                Ident::rec(recognized)
                    .map(Self::Ident)
                    .map_err(ItemDiag::Ident)
            })
            .map(|v| {
                *recognized = Recognized::default();
                v
            })
    }
}

#[derive(Clone)]
struct NamedBlock(Ident, Block);
#[derive(Clone)]
enum NamedBlockDiag {
    Ident(IdentDiag),
    Block(BlockDiag),
}

impl RecognizeParse for NamedBlock {}
impl Parse for NamedBlock {
    type Diag = NamedBlockDiag;
    fn parse(recognized: &mut Recognized) -> Result<Self, Self::Diag> {
        Ok(Self(
            Ident::rec(recognized).map_err(NamedBlockDiag::Ident)?,
            Block::rec(recognized).map_err(NamedBlockDiag::Block)?,
        ))
    }
}

#[derive(Clone)]
struct Block;
#[derive(Clone)]
enum BlockDiag {}

impl RecognizeParse for Block {}
impl Parse for Block {
    type Diag = BlockDiag;
    fn parse(recognized: &mut Recognized) -> Result<Self, Self::Diag> {
        Ok(Self)
    }
}

#[derive(Clone)]
struct Ident;
#[derive(Clone)]
enum IdentDiag {}

impl RecognizeParse for Ident {}
impl Parse for Ident {
    type Diag = IdentDiag;
    fn parse(recognized: &mut Recognized) -> Result<Self, Self::Diag> {
        Ok(Self)
    }
}

#[test]
fn test() {
    let mut recognized = Recognized::default();
    let res = NamedBlock::parse(&mut recognized);
    assert!(res.is_ok());
}
