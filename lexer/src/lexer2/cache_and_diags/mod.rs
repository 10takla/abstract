pub mod diag;

use diag::Diag;
use super::{Construct, ConstructItem, Pos};
use std::collections::HashMap;
use std_reset::prelude::Deref;

#[derive(Clone, Default, Debug)]
pub struct CacheAndDiags {
    pub(super) cursor: Option<Pos>,
    pub(super) cache: Cache,
    pub errors: Vec<Diag>,
    pub(super) warnings: PosS<Vec<Construct>>,
}

#[derive(Clone, Default, Debug)]
pub(super) struct Cache {
    // записываем только в const_item
    pub(super) pass: Vec<PassList>,
    pub(super) fails: HashMap<(Construct, Pos), Diag>,
}

#[derive(Clone, Default, Debug, Deref)]
pub(super) struct PassList {
    pub(super) index: usize,
    #[deref]
    pub(super) items: Vec<Pass>,
}

type Pass = (Construct, Pos, ConstructItem);

impl PassList {
    pub fn new(items: Vec<Pass>) -> Self {
        Self {
            index: Default::default(),
            items,
        }
    }
    pub fn get(&self) -> &(Construct, Pos, ConstructItem) {
        &self.items[self.index]
    }
}

type Warnings = PosS<Vec<Construct>>;

#[derive(Clone, Debug)]
pub struct PosS<T> {
    pub(super) cursor: Option<Pos>,
    pub(super) data: T,
}

impl<T> Default for PosS<Vec<T>> {
    fn default() -> Self {
        Self {
            cursor: Default::default(),
            data: Default::default(),
        }
    }
}
