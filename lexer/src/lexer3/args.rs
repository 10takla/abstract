use super::wrapper::CommonError;
use crate::lexer2::print::Print;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    str::CharIndices,
};

#[derive(Clone, Debug)]
pub struct Ctxt<'a> {
    pub code: Code<'a>,
    pub logger: Box<(Print, RefCell<usize>)>,
    pub errors: RefCell<Vec<CommonError>>,
    pub cache: Rc<RefCell<Cache>>,
}

pub type Cache = HashMap<(usize, TypeId), Box<dyn Any>>;

impl<'a> From<&'a str> for Ctxt<'a> {
    fn from(source: &'a str) -> Self {
        Self {
            code: source.into(),
            logger: Default::default(),
            errors: Default::default(),
            cache: Default::default(),
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
