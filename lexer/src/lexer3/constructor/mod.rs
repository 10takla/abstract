use super::*;
use lexer3_macros::{constructor, EnumRecog, Spanable};
use std::str::CharIndices;

constructor!(
    Item { FnC | StructC | TraitC | ImplV | AnyBlock | ConstC | AssignExpr | Literal | Ident }
            TraitC (Trait Ident MethodsC) { join Ignore }
                MethodsC (OpenFigureBracket MethodsI CloseFigureBracket) { join Ignore }
                    MethodsI <FnHead> { break CloseFigureBracket }
            ImplV { ImplFor | ImplC }
                ImplFor (Impl Ident For Ident ImplItemsC) { join Ignore }
                ImplC (Impl Ident ImplItemsC) { join Ignore }
                    ImplItemsC (OpenFigureBracket ImplItemsI CloseFigureBracket) { join Ignore }
                        ImplItemsI <ImplItemsV> { break CloseFigureBracket }
                            ImplItemsV { ConstC | FnC }
            StructC (Struct Ident Args) { join Ignore }
            FnC (FnHead Block) { join Ignore }
                FnHead (Fn Ident Args) { join Ignore }
            AnyBlock { NamedDistrBlock | DistrBlock | NamedBlock | Block }
                NamedDistrBlock (NamedBlock Distribution) { join Ignore }
                DistrBlock (Path Distribution) { join Ignore }
                NamedBlock (Ident Block) { join Ignore }
                Block (OpenFigureBracket BlockItems CloseFigureBracket) { join Ignore }
                    BlockItems <Item> { break CloseFigureBracket }
            ConstC (Const Assign) { join Ignore }
            AssignExpr { AssignAnd | Assign }
                AssignAnd (IdentAndType OpEq Literal) { join Ignore }
                    OpEq (Op Eq) { join Ignore }
                Assign (IdentAndType Eq Literal) { join Ignore }
            Literal { String | Number }
        Args { StructArgsC | TupleType }
            StructArgsC (OpenFigureBracket StructArgsI CloseFigureBracket)
                StructArgsI <IdentAndTypeC> {break CloseFigureBracket}
        Ignore <IgnoreV> {}
            IgnoreV { WhiteSpace | NextLine | Tab }
                WhiteSpace r" +"
                NextLine r"\n"
                Tab r"\t"
        Op { Add | Sub | Mul | Div }
            Add r#"\+"#
            Sub r#"-"#
            Mul r"\*"
            Div r#"/"#
        IdentAndType { IdentAndTypeC | Ident }
            IdentAndTypeC (Ident Colon Type) { join Ignore }
        Type { TupleType | BaseType }
            TupleType (OpenRoundBracket TupleTypeI CloseRoundBracket) { join Ignore }
                TupleTypeI <Type> { break CloseRoundBracket }
            BaseType { AnnotededTypeC | Ident }
                AnnotededTypeC (Ident OpenAngleBracket AnnotededTypeI CloseAngleBracket) { join Ignore }
                    OpenAngleBracket "<"
                    CloseAngleBracket ">"
                    AnnotededTypeI <AnnotededType> { break CloseAngleBracket }
                        AnnotededType { EqType | Ident }
                            EqType (Ident Eq Type) { join Ignore }
        Path { CurrenPath | EndPath | Ident }
            CurrenPath (CurrentPathV EndPath) { join Ignore }
                CurrentPathV { Self_ | Super | Crate | Ident }
                EndPath { WithItemsEnd | IdentPath }
                    WithItemsEnd (IdentPath NameSpace PathItemEnd) { join Ignore }
                        PathItemEnd { GlobImport | PathItemsC }
                            PathItemsC (OpenFigureBracket PathItemsI CloseFigureBracket) { join Ignore }
                                PathItemsI <PathItemC> { break CloseFigureBracket join Ignore }
                                    PathItemC (PathItemV Comma)
                                        PathItemV { Self_ | Super | GlobImport | Ident }
                                            GlobImport r#"\*"#
                    IdentPath <PathEl> {}
                        PathEl (NameSpace Ident) { join Ignore }
    // keywords
    Fn "fn"
    Const "const"
    Struct "struct"
    Trait "trait"
    Let "let"
    Impl "impl"
    Crate "crate"
    For "for"
    Self_ "self"
    Super "super"

    // other
    Number r"\b\d+\b"

    Distribution r#"\.\."#
    NameSpace "::"
    OpenFigureBracket r#"\{"#
    CloseFigureBracket r#"}"#
    OpenRoundBracket r#"\("#
    CloseRoundBracket r#"\)"#
    Eq r#"="#

    Comma ","
    Colon ":"
);

mod tokens {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub struct IdentMarker;
    pub type Ident = Token<IdentMarker>;

    impl TokenRecog for Ident {
        type Inner = IdentMarker;
        fn start_string_aware_recog(code: &str) -> Result<Slice, &'static str> {
            let mut iter = code.char_indices();

            let (i, char) = iter.next().ok_or("LineOver")?;

            let start_rule = |char: char| char.is_alphabetic() || char == '_';
            if let Some(start) = start_rule(char).then_some(i) {
                if start == code.len() - 1 {
                    Ok(start..start + 1)
                } else {
                    let end = iter
                        .find_map(|(i, char)| {
                            (!(start_rule(char) || char.is_digit(10))).then_some(i - 1)
                        })
                        .unwrap_or_else(|| code.len() - 1);
                    Ok(start..end + 1)
                }
            } else {
                if i == code.len() - 1 {
                    Err("StartsWithAlphabetic")
                } else {
                    let end = iter
                        .find_map(|(i, char)| {
                            (!(start_rule(char) || char.is_digit(10))).then_some(i - 1)
                        })
                        .unwrap_or_else(|| code.len() - 1);
                    Err("Alphabetic")
                }
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct StringMarker;
    pub type String = Token<StringMarker>;

    impl TokenRecog for String {
        type Inner = StringMarker;
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
}
use tokens::*;

mod items {
    use super::*;
    use crate::lexer2::NameSpace;
    use macros::parse_test;

    #[test]
    fn path() {
        macro_rules! check {
            ($l:literal, $($type_:tt)*) => {
                assert!(
                    match dbg!(Path::recog(&$l.into())) {
                        $($type_)* => true,
                        _ => false,
                    }
                );
            };
        }

        check!(
            "crate::sdfsdf::sdfsdf",
            Ok(Path::CurrenPath((CurrentPathV::Crate(_), ..)))
        );
        check!(
            "self::sdfsdf::sdfsdf",
            Ok(Path::CurrenPath((CurrentPathV::Self_(_), ..)))
        );
        check!(
            "super::sdfsdf::sdfsdf",
            Ok(Path::CurrenPath((CurrentPathV::Super(_), ..)))
        );
        check!(
            "sdfsdf::sdfsdf::sdfsfdf",
            Ok(Path::CurrenPath((CurrentPathV::Ident(_), ..)))
        );
        check!("::sdfsdf::sdfsdf", Ok(Path::EndPath(_)));

        check!(
            "sdfsdf::sdfsdf::*",
            Ok(Path::CurrenPath((
                _,
                _,
                EndPath::WithItemsEnd((_, _, _, _, PathItemEnd::GlobImport(_)))
            )))
        );

        check!(
            "sdfsdf::sdfsdf::{ *, }",
            Ok(Path::CurrenPath(
                (_, _, EndPath::WithItemsEnd((
                        _,
                        _,
                        _,
                        _,
                        PathItemEnd::PathItemsC((_, _, (vec), _, _))
                    )
                ))
            )) if let (PathItemV::GlobImport(_), ..) = vec[0]
        );
        check!(
            "sdfsdf::sdfsdf::{ self, }",
            Ok(Path::CurrenPath(
                (_, _,EndPath::WithItemsEnd((
                        _,_,
                        _,_,
                        PathItemEnd::PathItemsC((_, _,(vec),_, _))
                    )
                ))
            )) if let (PathItemV::Self_(_), ..) = vec[0]
        );
        check!(
            "sdfsdf::sdfsdf::{ sdfsdf, }",
            Ok(Path::CurrenPath(
                (_, _,EndPath::WithItemsEnd((
                        _,_,
                        _,_,
                        PathItemEnd::PathItemsC((_,_, (vec),_, _))
                    )
                ))
            )) if let (PathItemV::Ident(_), ..) = vec[0]
        );
    }
}
