use super::wrapper::CommonError;
use crate::lexer2::print::Print;
use std::{cell::RefCell, rc::Rc, str::CharIndices};

#[derive(Clone, Debug)]
pub struct Ctxt<'a> {
    pub code: Code<'a>,
    pub logger: Box<(Print, RefCell<usize>)>,
    pub errors: RefCell<Vec<CommonError>>,
}

impl<'a> From<&'a str> for Ctxt<'a> {
    fn from(source: &'a str) -> Self {
        Self {
            code: source.into(),
            logger: Default::default(),
            errors: Default::default(),
        }
    }
}

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
