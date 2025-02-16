use super::*;
use macros::constructor;

constructor!(
    tokens {
        Ident [StartsWithAlphabetic Alphabetic] {
            // r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"
            let start_rule = |char: char| char.is_alphabetic() || char == '_';

            let mut iter = arg.code.iter();
            let &(i, char) = iter.next().ok_or((arg.code.cursor..=arg.code.cursor, ErrorType::LineOver))?;

            if let Some(start) = start_rule(char).then_some(i) {
                if start == arg.code.source.len() - 1 {
                    Ok(start..=start)
                } else {
                    let end = iter.find_map(|&(i, char)| {
                            (!(start_rule(char) || char.is_digit(10))).then_some(i-1)
                        })
                        .unwrap_or_else(|| {
                            arg.code.source.len() - 1
                        });
                    Ok(start..=end)
                }
            } else {
                if i == arg.code.source.len() - 1 {
                    Err((i..=i, ErrorType::Ident(IdentError::StartsWithAlphabetic)))
                } else {
                    let end = iter.find_map(|&(i, char)| {
                            (!( start_rule(char) || char.is_digit(10))).then_some(i - 1)
                        })
                        .unwrap_or_else(|| {
                            arg.code.source.len() - 1
                        });
                    Err((i..=end, ErrorType::Ident(IdentError::Alphabetic)))
                }
            }
        }
        String [StartsWithNumber StartsWithQuote EndsWithQuote] {
            // r#""[^"\\]*(?:\\.[^"\\]*)*""#
            let mut iter = arg.code.iter();

            let &(i, char) = iter.next().ok_or((arg.code.cursor..=arg.code.cursor, ErrorType::LineOver))?;
            let start = (char == '"').then_some(i).ok_or((i..=i, ErrorType::String(StringError::StartsWithQuote)))?;

            for &(i, char) in iter.clone() {
                if char == '"' {
                    return Ok(start..=i);
                }
            }

            let tmp = iter.last().unwrap().0;

            Err((tmp..=tmp, ErrorType::String(StringError::EndsWithQuote)))
        }

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
    }
    items { 
        Items ! (Item) (Ignore)
    }
    common {
        Item -> FnC | StructC | TraitC | ImplV | AnyBlock | ConstC | AssignExpr | Literal | Ident
            TraitC (Ignore) -> Trait Ident MethodsC
                MethodsC (Ignore) -> OpenFigureBracket MethodsI CloseFigureBracket
                    MethodsI ! (FnHead) (Ignore) CloseFigureBracket
            ImplV -> ImplFor | ImplC
                ImplFor (Ignore) -> Impl Ident For Ident ImplItemsC
                ImplC (Ignore) -> Impl Ident ImplItemsC
                    ImplItemsC (Ignore) -> OpenFigureBracket ImplItemsI CloseFigureBracket
                        ImplItemsI ! (ImplItemsV) (Ignore) CloseFigureBracket
                            ImplItemsV -> ConstC | FnC
            StructC (Ignore) -> Struct Ident Args
            FnC (Ignore) -> FnHead Block
                FnHead (Ignore) -> Fn Ident Args
            AnyBlock -> NamedDistrBlock | DistrBlock | NamedBlock | Block
                NamedDistrBlock (Ignore) -> NamedBlock Distribution
                DistrBlock (Ignore) -> Path Distribution
                NamedBlock (Ignore) -> Ident Block
                Block -> OpenFigureBracket BlockItems CloseFigureBracket
                    BlockItems ! (Item) (Ignore) CloseFigureBracket
            ConstC (Ignore) -> Const Assign
            AssignExpr -> AssignAnd | Assign
                AssignAnd (Ignore) -> IdentAndType OpEq Literal
                    OpEq -> Op Eq
                Assign (Ignore) -> IdentAndType Eq Literal
            Literal -> String | Number
        Args -> StructArgsC | TupleType
            StructArgsC (Ignore) -> OpenFigureBracket StructArgsI CloseFigureBracket
                StructArgsI ! (IdentAndTypeC) (Ignore) CloseFigureBracket
        Ignore (IgnoreV) #
            IgnoreV -> WhiteSpace | NextLine | Tab
                WhiteSpace r" +"
                NextLine r"\n"
                Tab r"\t"
        Op -> Add | Sub | Mul | Div
            Add r#"\+"#
            Sub r#"-"#
            Mul r"\*"
            Div r#"/"#
        IdentAndType -> IdentAndTypeC | Ident
            IdentAndTypeC (Ignore) -> Ident Colon Type
        Type -> TupleType | BaseType
            TupleType (Ignore) -> OpenRoundBracket TupleTypeI CloseRoundBracket
                TupleTypeI ! (Type) (Ignore) CloseRoundBracket
            BaseType -> AnnotededTypeC | Ident
                AnnotededTypeC (Ignore) -> Ident OpenAngleBracket AnnotededTypeI CloseAngleBracket
                    OpenAngleBracket "<"
                    CloseAngleBracket ">"
                    AnnotededTypeI ! (AnnotededType) (Ignore) CloseAngleBracket
                        AnnotededType -> EqType | Ident
                            EqType (Ignore) -> Ident Eq Type!
        Path -> CurrenPath | EndPath | Ident
            CurrenPath (Ignore) -> CurrentPathV EndPath
                CurrentPathV -> Self_ | Super | Crate | Ident
                EndPath -> WithItemsEnd | IdentPath
                    IdentPath ! (PathEl) #
                            PathEl (Ignore) -> NameSpace Ident
                    WithItemsEnd (Ignore) ->  IdentPath NameSpace PathItemEnd
                        PathItemEnd -> GlobImport | PathItemsC
                            PathItemsC (Ignore) -> OpenFigureBracket PathItemsI CloseFigureBracket
                                PathItemsI ! (PathItemC) (Ignore) CloseFigureBracket
                                    PathItemC (Ignore) -> PathItemV Comma
                                        PathItemV -> Self_ | Super | GlobImport | Ident
                                            GlobImport r#"\*"#
        // Keywords
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
    }
    common {
        Tmp -> Ident Eq
    }
    common {
        CacheConstructItem -> Var1 | Var2
        Var1 -> Ident Ignore Eq Number WhiteSpace NamedDistrBlock Item Ignore AssignAnd WhiteSpace AssignExpr Literal Sub OpEq Op Number
        Var2 -> Ident Ignore Eq Number WhiteSpace NamedDistrBlock Item Ignore AssignAnd WhiteSpace AssignExpr Literal Sub OpEq Op
    }
    common {
        CacheConstructHead -> Var3 | CommCons1
        CacheToken -> CommCons1 | Ident
        CacheEnum -> Var5 | Op
        CacheConstructWalkthroug -> Var6 | Var7 | Var8

        CommCons1 -> Ident Distribution
        Var3 -> CommCons1 Ident
        Var5 -> Op Ident
        Var6 -> Ident CloseFigureBracket Distribution
        Var7 -> Ident CloseFigureBracket OpenFigureBracket CloseFigureBracket
        Var8 -> Ident CloseFigureBracket OpenFigureBracket WhiteSpace
    }
);