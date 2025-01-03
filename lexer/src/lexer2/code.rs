use super::Pos;
use std::sync::Arc;
use std_reset::prelude::Deref;

#[derive(Clone, Debug)]
pub(super) struct Code {
    pub source: Source,
    pub cursor: Pos,
}

impl<'a> IntoIterator for &'a Code {
    type Item = &'a (usize, char);
    type IntoIter = std::slice::Iter<'a, (usize, char)>;

    fn into_iter(self) -> Self::IntoIter {
        self.source[self.cursor..].iter()
    }
}

impl Code {
    pub(super) fn get_current(&self) -> (usize, char) {
        self.source[self.cursor]
    }

    pub(super) fn t(&self) -> std::string::String {
        self.source
            .as_ref()
            .into_iter()
            .skip_while(|&&(i, _)| i < self.cursor)
            .map(|(_, v)| v)
            .collect()
    }

    pub(super) fn len(&self) -> usize {
        self.source.len()
    }

    pub(super) fn iter(&self) -> std::slice::Iter<'_, (usize, char)> {
        self.source[self.cursor..].into_iter()
    }
}

#[derive(Clone, Debug, Deref)]
pub struct Source {
    pub real_source: std::string::String,
    #[deref]
    source: Arc<Vec<(usize, char)>>,
}

impl Source {
    pub fn new(source: &str) -> Self {
        Self {
            real_source: source.into(),
            source: Arc::new(source.chars().enumerate().collect()),
        }
    }
}
