use std::{fmt::Display, ops::RangeInclusive};

use super::Code;

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Slice<'s> {
    pub range: RangeInclusive<usize>,
    pub source: &'s str,
}

impl Slicable for Slice<'_> {
    fn get_slice(&self) -> RangeInclusive<usize> {
        self.range.clone()
    }
}

impl Display for Slice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl<'s> Slice<'s> {
    pub fn new(range: RangeInclusive<usize>, code: &Code<'s>) -> Self {
        Self {
            range: range.clone(),
            source: code.get_slice(range).unwrap(),
        }
    }
}

pub trait Slicable {
    fn get_slice(&self) -> RangeInclusive<usize>;
    fn get_start(&self) -> usize {
        *self.get_slice().start()
    }
    fn get_end(&self) -> usize {
        *self.get_slice().end()
    }
}
