pub mod diag;

use super::{Construct, ConstructItem, Op, ParseArgs, Pos};
use diag::Diag;
use std::{collections::HashMap, iter::once};
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

impl Cache {
    pub(super) fn clear(&mut self) {
        self.pass.clear();
        self.fails.clear();
    }

    pub(super) fn check(self, e: Diag) -> Diag {
        self.fails
            .into_values()
            .into_iter()
            .chain(once(e))
            .max_by(|a, b| a.slice.end().cmp(b.slice.end()))
            .unwrap()
    }
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
    pub fn get(&self) -> &Pass {
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
