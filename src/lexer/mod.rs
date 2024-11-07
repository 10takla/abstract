pub mod items;

use colored::Colorize;
use std::fmt::{Debug, Display};

const IGNORE: [char; 3] = [' ', '\n', '\t'];

#[derive(PartialEq, Debug, Clone)]
pub struct Code {
    source: &'static str,
    cursor: usize,
}

impl Code {
    pub const fn new(source: &'static str) -> Self {
        Self { source, cursor: 0 }
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, char)> + Clone + Debug {
        let cursor = self.cursor;
        self.source
            .char_indices()
            .skip_while(move |&(i, _)| i < cursor)
    }

    pub fn consume(&mut self, skip_index: usize) -> &mut Self {
        self.cursor = skip_index + 1;
        self
    }

    pub fn offset(&mut self, count: usize) -> &mut Self {
        self.cursor += count;
        self
    }

    pub fn end(&mut self, like_end: &impl Slicable) -> &mut Self {
        self.consume(like_end.get_end())
    }

    pub fn len(&self) -> usize {
        self.source.len()
    }

    pub fn get_slice(&self, offset: usize) -> Option<&str> {
        self.source.get(self.cursor..self.cursor + offset)
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            &self.source[..self.cursor],
            self.source[self.cursor..=self.cursor].underline(),
            &self.source[self.cursor + 1..]
        )
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Slice {
    pub start_end: [usize; 2],
    pub source: &'static str,
}

impl Slicable for Slice {
    fn get_end(&self) -> usize {
        self.start_end[1]
    }
}

impl Display for Slice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl Slice {
    pub fn new(start_end: [usize; 2], code: &Code) -> Self {
        Self {
            start_end,
            source: &code.source[start_end[0]..=start_end[1]],
        }
    }
}

pub trait Parse: Slicable + Sized {
    fn parse(code: &Code) -> Option<Self>;

    fn parse_and_consume(code: &mut Code) -> Option<Self> {
        Self::parse(code).map(|v| {
            code.end(&v);
            v
        })
    }
}

pub trait Slicable {
    fn get_end(&self) -> usize;
}

fn check<T: Parse + PartialEq + Debug>(code: &'static str, b: fn(&Code) -> T) {
    let code = &mut Code::new(code);
    assert_eq!(T::parse(code), Some(b(code)));
}

fn check_none<T: Parse + PartialEq + Debug>(code: &'static str) {
    assert_eq!(T::parse(&mut Code::new(code)), None);
}

#[macro_export]
macro_rules! parse_variants {
    ($( $expr:expr ), + $(,)?) => {
        [
            $(
                Box::new(|| $expr) as Box<dyn Fn() -> Option<Self>>,
            )+
        ]
        .iter()
        .find_map(|parse_fn| parse_fn())
    };
    ($code:ident $( $from:ty => $to:path ), + $(,)?) => {
        crate::parse_variants!(
            $(
                <$from>::parse($code).map($to),
            )+
        )
    };
}
