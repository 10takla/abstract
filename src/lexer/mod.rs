pub mod items;
pub mod new_cursor;

use colored::Colorize;
use std::{
    fmt::{Debug, Display},
    iter::Enumerate,
    ops::RangeInclusive,
    str::Lines,
};

const IGNORE: [char; 3] = [' ', '\n', '\t'];

#[derive(PartialEq, Debug, Clone)]
pub struct Code {
    pub source: &'static str,
    byte_indices: Vec<usize>,
    pub cursor: usize,
}

impl Code {
    pub fn new(source: &'static str) -> Self {
        Self {
            source,
            cursor: 0,
            byte_indices: source.char_indices().map(|(idx, _)| idx).collect(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, char)> + '_ + Clone {
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

    pub fn get_slice(&self, offset: usize) -> Option<&str> {
        self.byte_indices
            .get(self.cursor + offset - 1)
            .map(|v| v)
            .and_then(|v| self.source.get(self.byte_indices[self.cursor]..=*v))
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.byte_indices.is_empty() {
            let cursor_byte_index = self.byte_indices[self.cursor];
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
            write!(f, "",)
        }
    }
}

#[test]
#[ignore]
fn new_cursor() {
    let mut code = Code::new("a◕◕fadф");
    code.offset(2);
    println!("{code}");
    code.consume(2);
    println!("{code}");
    dbg!(code.get_slice(3));
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Slice {
    pub range: RangeInclusive<usize>,
    pub source: &'static str,
}

impl Slicable for Slice {
    fn get_start(&self) -> usize {
        *self.range.start()
    }
    fn get_end(&self) -> usize {
        *self.range.end()
    }
}

impl Display for Slice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl Slice {
    pub fn new(range: RangeInclusive<usize>, code: &Code) -> Self {
        Self {
            range: range.clone(),
            source: &code.source[range],
        }
    }
}

pub trait Parse: Slicable + Sized + Debug {
    fn parse(code: &Code) -> Option<Self>;

    fn parse_and_consume(code: &mut Code) -> Option<Self> {
        Self::parse(code).map(|v| {
            code.end(&v);
            v
        })
    }
}

pub trait Slicable {
    fn get_start(&self) -> usize;
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
