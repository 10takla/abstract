use super::*;
use lexer3_macros::Spanable;
use std::str::CharIndices;

#[derive(Debug, PartialEq)]
pub struct Ident;

impl RegularToken for Ident {
    const REG_EXPR: &'static str = r"\b[_a-zA-Z][_a-zA-Z0-9]*\b";
}

#[derive(Debug, PartialEq)]
pub struct String;

impl TokenRecog for Token<String> {
    type Inner = String;
    fn start_string_aware_recog(code: &str) -> Result<Slice, &'static str> {
        let mut iter = code.char_indices();

        let (i, char) = iter.next().ok_or("LineOver")?;
        let start = (char == '"').then_some(i).ok_or("StartsWithQuote")?;

        for (i, char) in iter.clone() {
            if char == '"' {
                return Ok(start..i + 1);
            }
        }

        let tmp = iter.last().unwrap().0;

        Err("EndsWithQuote")
    }
}

#[derive(Debug, PartialEq, Spanable)]
pub enum Item {
    Ident(Token<Ident>),
    String(Token<String>),
}

impl EnumRecog for Item {
    type Output = Self;
    const N: usize = 2;
    
    fn structure_assembling<'a>(
        code: &'a Code,
    ) -> [Box<dyn core::ops::Fn() -> Result<Self::Output, &'static str> + 'a>; Self::N] {
        [
            Box::new(|| Token::<Ident>::recog(code).map(Self::Ident)),
            Box::new(|| Token::<String>::recog(code).map(Self::String)),
        ]
    }
}

#[derive(Debug, PartialEq, Spanable)]
pub struct IdentString(pub Token<Ident>, pub Token<String>, pub Token<Ident>);

impl SequenceRecog for IdentString {
    type Output = Self;
    fn structure_assembling(code: &mut Code) -> Result<Self::Output, &'static str> {
        Ok(Self(
            Self::promotion::<Token<Ident>>(code)?,
            Self::promotion::<Token<String>>(code)?,
            Self::promotion::<Token<Ident>>(code)?,
        ))
    }
}

// pub struct IdentStringMarker;

// impl MarkerConversion for IdentStringMarker {
//     type Output = IdentString;
// }
