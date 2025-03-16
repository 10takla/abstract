use super::*;
use macros::Spanable;
use std::{any::TypeId, str::CharIndices};

#[derive(Debug, PartialEq, Clone)]
pub struct Ident;

impl RegularToken for Ident {
    const REG_EXPR: &'static str = r"\b[_a-zA-Z][_a-zA-Z0-9]*\b";
}

#[derive(Debug, PartialEq, Clone)]
pub struct String;

impl TokenRecog for Token<String> {
    type Inner = String;
    fn start_string_aware_recog(code: &str) -> Result<Slice, TokenError> {
        let mut iter = code.char_indices();

        let (i, char) = iter.next().ok_or(TokenError::LineOver)?;
        let start = (char == '"')
            .then_some(i)
            .ok_or(TokenError::CommonTokenError(
                0..1,
                CommonTokenError::CurrentErrors("StartsWithQuote"),
            ))?;

        for (i, char) in iter.clone() {
            if char == '"' {
                return Ok(start..i + 1);
            }
        }

        let tmp = iter.last().unwrap().0;

        Err(TokenError::CommonTokenError(
            tmp..tmp + 1,
            CommonTokenError::CurrentErrors("EndsWithQuote"),
        ))
    }
}

#[derive(Debug, PartialEq, Spanable, Clone)]
pub enum Item {
    Ident(Token<Ident>),
    String(Token<String>),
}

impl EnumRecog for Item {
    type Output = Self;
    fn structure_assembling<'a>(
        ctxt: &'a Ctxt,
    ) -> Vec<Box<dyn core::ops::Fn() -> Result<Self::Output, CommonError> + 'a>> {
        vec![
            Box::new(|| <Token<Ident>>::recog(ctxt).map(Self::Ident)),
            Box::new(|| <Token<String>>::recog(ctxt).map(Self::String)),
        ]
    }
}

#[derive(Debug, PartialEq, Spanable, Clone)]
pub struct IdentString(pub Token<Ident>, pub Token<String>, pub Token<Ident>);

impl SequenceRecog for IdentString {
    type Output = Self;
    fn structure_assembling(ctxt: &mut Ctxt) -> Result<Self::Output, CommonError> {
        Ok(Self(
            Self::promotion::<Token<Ident>>(ctxt)?,
            Self::promotion::<Token<String>>(ctxt)?,
            Self::promotion::<Token<Ident>>(ctxt)?,
        ))
    }
}

// pub struct IdentStringMarker;

// impl MarkerConversion for IdentStringMarker {
//     type Output = IdentString;
// }

struct IdentCache;

// impl Cachable for Token<Ident> {
//     fn get_cache(&self) {

//     }
// }

// fn tmp() {
//     <Token<Ident>>::check_cache(&"".into(), &vec![TypeId::of::<Token<Ident>>()]);
// }
