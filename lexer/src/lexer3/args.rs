use std::{cell::RefCell, rc::Rc, str::CharIndices};

#[derive(Clone, Debug)]
pub struct Code<'a> {
    pub cursor: usize,
    pub source: Source<'a>,
}

impl<'a> Code<'a> {
    pub(super) fn iter(&'a self) -> impl Iterator<Item = (usize, char)> + Clone + 'a {
        self.source[self.cursor..]
            .char_indices()
            .map(|(i, v)| (i + self.cursor, v))
    }
}

pub type Source<'a> = &'a str;

impl<'a> From<&'a str> for Code<'a> {
    fn from(source: &'a str) -> Self {
        Self {
            cursor: Default::default(),
            source,
        }
    }
}
