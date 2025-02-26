use super::*;
use lexer3_macros::{constructor, EnumRecog, Spanable};
use std::str::CharIndices;

constructor!(
    Item { FnC | StructC | TraitC | ImplV | AnyBlock | ConstC | AssignExpr | Literal | Ident }
            TraitC (Trait Ident MethodsC)
                MethodsC (OpenFigureBracket MethodsI CloseFigureBracket)
                    MethodsI <FnHead> { break CloseFigureBracket }
            ImplV { ImplFor | ImplC }
                ImplFor (Impl Ident For Ident ImplItemsC)
                ImplC (Impl Ident ImplItemsC)
                    ImplItemsC (OpenFigureBracket ImplItemsI CloseFigureBracket)
                        ImplItemsI <ImplItemsV> { break CloseFigureBracket }
                            ImplItemsV { ConstC | FnC }
            StructC (Struct Ident Args)
            FnC (FnHead Block)
                FnHead (Fn Ident Args)
            AnyBlock { NamedDistrBlock | DistrBlock | NamedBlock | Block }
                NamedDistrBlock (NamedBlock Distribution)
                DistrBlock (Path Distribution)
                NamedBlock (Ident Block)
                Block (OpenFigureBracket BlockItems CloseFigureBracket)
                    BlockItems <Item> { break CloseFigureBracket }
            ConstC (Const Assign)
            AssignExpr { AssignAnd | Assign }
                AssignAnd (IdentAndType OpEq Literal)
                    OpEq (Op Eq)
                Assign (IdentAndType Eq Literal)
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
            IdentAndTypeC (Ident Colon Type)
        Type { TupleType | BaseType }
            TupleType (OpenRoundBracket TupleTypeI CloseRoundBracket)
                TupleTypeI <Type> { break CloseRoundBracket }
            BaseType { AnnotededTypeC | Ident }
                AnnotededTypeC (Ident OpenAngleBracket AnnotededTypeI CloseAngleBracket)
                    OpenAngleBracket "<"
                    CloseAngleBracket ">"
                    AnnotededTypeI <AnnotededType> { break CloseAngleBracket }
                        AnnotededType { EqType | Ident }
                            EqType (Ident Eq Type)
        Path { CurrenPath | EndPath | Ident }
            CurrenPath (CurrentPathV EndPath)
                CurrentPathV { Self_ | Super | Crate | Ident }
                EndPath { WithItemsEnd | IdentPath }
                    IdentPath <PathEl> {}
                            PathEl (NameSpace Ident)
                    WithItemsEnd (IdentPath NameSpace PathItemEnd)
                        PathItemEnd { GlobImport | PathItemsC }
                            PathItemsC (OpenFigureBracket PathItemsI CloseFigureBracket)
                                PathItemsI <PathItemC> { break CloseFigureBracket }
                                    PathItemC (PathItemV Comma)
                                        PathItemV { Self_ | Super | GlobImport | Ident }
                                            GlobImport r#"\*"#
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
