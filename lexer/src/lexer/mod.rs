pub mod diag;
pub mod items;

use colored::Colorize;
use std::{
    fmt::{Debug, Display},
    ops::RangeInclusive,
    rc::Rc,
};

const IGNORE: [char; 3] = [' ', '\n', '\t'];

#[derive(PartialEq, Debug, Clone)]
pub struct Code<'s> {
    pub source: &'s str,
    byte_indices: Rc<Vec<usize>>,
    pub cursor: usize,
}

impl<'s> Code<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            cursor: 0,
            byte_indices: Rc::new(source.char_indices().map(|(idx, _)| idx).collect()),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, char)> + '_ + Clone{
        self.byte_indices[self.cursor..]
            .iter()
            .enumerate()
            .map(|(i, &byte_id)| {
                (
                    i + self.cursor,
                    self.source[byte_id..].chars().next().unwrap(),
                )
            })
    }

    pub fn set_cursor(&mut self, cursor: usize) -> &mut Self {
        self.cursor = cursor;
        self
    }

    pub fn consume(&mut self, skip_index: usize) -> &mut Self {
        if skip_index + 1 > self.cursor {
            self.set_cursor(skip_index + 1)
        } else {
            panic!("current cursor position: {}", self.cursor);
        }
    }

    pub fn offset(&mut self, count: usize) -> &mut Self {
        self.set_cursor(self.cursor + count)
    }

    pub fn end(&mut self, like_end: &impl Slicable) -> &mut Self {
        self.consume(like_end.get_end())
    }

    pub fn len(&self) -> usize {
        self.byte_indices.len()
    }

    pub fn get_offset_slice(&self, offset: usize) -> Option<&str> {
        self.byte_indices
            .get(self.cursor + offset - 1)
            .cloned()
            .and_then(|end| self.source.get(self.byte_indices[self.cursor]..=end))
    }

    pub fn get_slice(&self, range: RangeInclusive<usize>) -> Option<&'s str> {
        self.byte_indices
            .get(*range.start())
            .zip(
                self.byte_indices
                    .get(*range.end() + 1)
                    .map(|v| v - 1)
                    .or(Some(self.source.len() - 1)),
            )
            .and_then(|(&start, end)| self.source.get(start..=end))
    }
}

impl Display for Code<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.byte_indices.is_empty() {
            if let Some(&cursor_byte_index) = self.byte_indices.get(self.cursor) {
                let next_byte_index = self
                    .byte_indices
                    .get(self.cursor + 1)
                    .map(|v| *v)
                    .unwrap_or(self.source.len());

                write!(
                    f,
                    "{}{}{}",
                    &self.source[..cursor_byte_index],
                    &self.source[cursor_byte_index..next_byte_index].underline(),
                    &self.source[next_byte_index..]
                )
            } else {
                write!(f, "{}", self.source)
            }
        } else {
            write!(f, "",)
        }
    }
}

#[test]
#[ignore]
fn code() {
    let mut code = Code::new("a◕◕fadф");
    code.offset(2);
    println!("{code}");
    code.consume(2);
    println!("{code}");
    dbg!(code.get_offset_slice(3));
}

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

pub trait Parse<'s>: Slicable + Sized {
    fn parse(code: &Code<'s>) -> Option<Self>;

    fn parse_and_consume(code: &mut Code<'s>) -> Option<Self> {
        Self::parse(code).map(|v| {
            code.end(&v);
            v
        })
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

fn check<'s, T: Parse<'s> + PartialEq + Debug>(
    source: &'s str,
    get_item: impl FnOnce(&Code<'s>) -> T,
) {
    let code = &mut Code::new(source);
    assert_eq!(T::parse(code), Some(get_item(code)));
}

fn check_none<'s, T: Parse<'s> + PartialEq + Debug>(code: &'s str) {
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
