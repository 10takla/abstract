use super::{items::shared::whitespaces::Whitespaces, Code, Parse, Slicable, Slice, IGNORE};
use macros::Slicable;
use std::fmt::Display;

type Diags<T> = Vec<Diag<T>>;
type Diag<T> = (usize, T);
pub trait DiagParse<'s>: Sized {
    type Diag;

    fn diag(code: &Code<'s>) -> Result<Self, Diags<Self::Diag>> {
        let mut diags = vec![];
        Self::parse(code, &mut diags).ok_or(diags)
    }

    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>) -> Option<Self>;
}

#[derive(PartialEq, Debug)]
enum Item<'s> {
    Number(Number<'s>),
    String(String<'s>),
}

#[derive(PartialEq, Debug)]
enum ItemDiags {
    Number(NumberDiags),
    String(StringDiags),
}

impl<'s> DiagParse<'s> for Item<'s> {
    type Diag = ItemDiags;

    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>) -> Option<Self> {
        [
            Box::new(|| {
                Number::diag(code).map(|v| Self::Number(v)).map_err(|v| {
                    v.into_iter()
                        .map(|(i, v)| (i, ItemDiags::Number(v)))
                        .collect()
                })
            }) as Box<dyn Fn() -> Result<Self, Diags<Self::Diag>>>,
            Box::new(|| {
                String::diag(code).map(|v| Self::String(v)).map_err(|v| {
                    v.into_iter()
                        .map(|(i, v)| (i, ItemDiags::String(v)))
                        .collect()
                })
            }),
        ]
        .into_iter()
        .find_map(|f| match f() {
            Ok(item) => Some(item),
            Err(v) => {
                diags.extend(v);
                None
            }
        })
    }
}

#[test]
fn diag_test() {
    let check = |source| {
        assert_eq!(Item::diag(&Code::new(source)), Err(vec![]));
    };

    check("  43c");
}

#[derive(PartialEq, Debug, Slicable)]
pub struct Number<'s>(pub Slice<'s>);

#[derive(PartialEq, Debug)]
pub enum NumberDiags {
    StartsWithNumber,
    MustBeNumber,
}

impl Display for NumberDiags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NumberDiags::*;
        write!(
            f,
            "{}",
            match self {
                StartsWithNumber => "Должно начинатся с числа",
                MustBeNumber => "Должно быть число",
            }
        )
    }
}

impl<'s> DiagParse<'s> for Number<'s> {
    type Diag = NumberDiags;

    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>) -> Option<Self> {
        let code = &mut code.clone();

        Whitespaces::parse_and_consume(code);
        let mut iter = code.iter();

        let (i, char) = iter.next()?;
        let start = if char.is_digit(10) {
            if i == code.len() - 1 {
                return Some(Self(Slice::new(i..=i, code)));
            }
            i
        } else {
            diags.push((i, NumberDiags::StartsWithNumber));
            return None;
        };

        let end = (|| {
            for (i, char) in iter.clone() {
                if IGNORE.contains(&char) {
                    return Some(i - 1);
                }
                if char.is_digit(10) {
                    if i == code.len() - 1 {
                        return Some(i);
                    }
                    continue;
                }
                diags.push((i, NumberDiags::MustBeNumber));
                return None;
            }
            None
        })()?;

        Some(Self(Slice::new(start..=end, code)))
    }
}

#[derive(Debug, PartialEq, Slicable)]
pub struct String<'s>(pub Slice<'s>);

#[derive(PartialEq, Debug)]
pub enum StringDiags {
    StartsWithQuote,
    EndsWithQuote,
}

impl Display for StringDiags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StringDiags::*;
        write!(
            f,
            "{}",
            match self {
                StartsWithQuote => "Должно начинатся с \"",
                EndsWithQuote => "Должно заканчиватся на \"",
            }
        )
    }
}

impl<'s> DiagParse<'s> for String<'s> {
    type Diag = StringDiags;

    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>) -> Option<Self> {
        let code = &mut code.clone();

        Whitespaces::parse_and_consume(code);
        let mut iter = code.iter();

        let (i, char) = iter.next()?;
        let start = (char == '"').then_some(i).or_else(|| {
            diags.push((i, StringDiags::StartsWithQuote));
            None
        })?;

        for (i, char) in iter {
            if char == '"' {
                return Some(Self(Slice::new(start..=i, code)));
            }
        }
        diags.push((code.cursor, StringDiags::EndsWithQuote));
        None
    }
}
