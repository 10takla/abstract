use crate::{parser::*, Ctxt, *};
use macros::{peg_grammar, RegularToken, Spanable};
use std::{fmt::Display, marker::PhantomData};

pub type ItemError = ErrorRecovery<Item>;

impl Description for Item {
    const DESCR: Descr = Descr {
        self_type: "Item",
        content: "Fn / Struct_ / Ident / WhiteSpaces",
    };
}

pub type NextGenericParamError = ErrorRecovery<NextGenericParam>;

impl Description for NextGenericParam {
    const DESCR: Descr = Descr {
        self_type: "NextGenericParam",
        content: r#"(I "," I GenericParam)"#,
    };
}

peg_grammar! {
    Items2 ::= I (Item I)*
    Items ::= (Item / ItemError)*
        Item ::= Fn / Struct_ / Ident / WhiteSpaces
            Fn ::= "fn" I Ident I (GenericParams I)? r"\(" I (FunctionParameters I)? r"\)" (I FunctionReturn)? I FunctionBody
                GenericParams ::= r"<" I GenericParamsBody? I r">"
                    GenericParamsBody ::= GenericParam NextGenericParam* (I ",")?
                        NextGenericParam ::= (I "," I GenericParam)
                        GenericParam ::= ConstParam / TypeParam
                            ConstParam ::= "const" I Ident I ":" I Type
                            TypeParam ::= Ident (I "=" I Type )?
                FunctionParameters ::= FunctionParam (I "," I FunctionParam)[* r"\)"] (I ",")?
                    FunctionParam ::= DefaultField / Field
                        DefaultField ::= Field I "=" I Value
                            @Field ::= (Ident I ":" I Type) / Ident
                FunctionReturn ::= "->" I Type
                FunctionBody ::= r"\{" I r"\}"
            Struct_ ::= "struct" I Ident I (GenericParams I)? r"\{" I (StructFields I)? r"\}"
                StructFields ::= StructParam (I "," I StructParam)* (I ",")?
                    StructParam ::= DefaultStructField / StructField
                        DefaultStructField ::= StructField I "=" I Value
                            @StructField ::= (Ident I ":" I Type) / Type
    Type ::= Ident
    Value ::= Ident / String

    // r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"
    String ::= BaseString / RawString / Character / Byte / ByteString / RawByteString / CString / RawCString
        BaseString ::= r#""[^"\\]*(?:\\.[^"\\]*)*""#
        RawString ::= r##"r#""[^"\\]*(?:\\.[^"\\]*)*"#"##
        ByteString ::= r#"b""[^"\\]*(?:\\.[^"\\]*)*""#
        RawByteString ::= r##"br#""[^"\\]*(?:\\.[^"\\]*)*"#"##
        CString ::=	r#"c""[^"\\]*(?:\\.[^"\\]*)*""#
        RawCString ::= r##"cr#""[^"\\]*(?:\\.[^"\\]*)*"#"##
        Character ::= r#"'.*'"#
        Byte ::= r#"b'.*'"#


    I ::= r"[\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u200E\u200F\u2028\u2029]*"
        WhiteSpaces ::= r"[\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u200E\u200F\u2028\u2029]+"

    FnKeyword ::= "fn"
    ConstKeyword ::= "const"
    StructKeyword ::= "struct"

}

mod tokens {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    pub struct IdentMarker;
    pub type Ident = Token<IdentMarker>;

    impl TokenRecog for Ident {
        type Inner = IdentMarker;
        fn start_string_aware_recog(code: &str) -> Result<Slice, TokenError> {
            let mut iter = code.char_indices();

            let (i, char) = iter.next().ok_or(TokenError::LineOver)?;

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
                    Err(TokenError::CommonTokenError(
                        i..i + 1,
                        CommonTokenError::CurrentErrors("StartsWithAlphabetic"),
                    ))
                } else {
                    let end = iter
                        .find_map(|(i, char)| {
                            (!(start_rule(char) || char.is_digit(10))).then_some(i - 1)
                        })
                        .unwrap_or_else(|| code.len() - 1);
                    Err(TokenError::CommonTokenError(
                        i..end + 1,
                        CommonTokenError::CurrentErrors("Alphabetic"),
                    ))
                }
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct StringMarker;
    pub type String = Token<StringMarker>;

    enum StringError {
        LineOver,
        StartsWithQuote,
        EndsWithQuote,
    }

    impl TokenRecog for String {
        type Inner = StringMarker;
        fn start_string_aware_recog(code: &str) -> Result<Slice, TokenError> {
            let mut iter = code.char_indices();

            let (i, ch) = iter.next().ok_or(TokenError::LineOver)?;
            let start = (ch == '"')
                .then_some(i)
                .ok_or(TokenError::CommonTokenError(
                    i..i + ch.len_utf8(),
                    CommonTokenError::CurrentErrors("StartsWithQuote"),
                ))?;

            for (i, char) in iter.clone() {
                if char == '"' {
                    return Ok(start..i + 1);
                }
            }

            let (i, ch) = iter.last().unwrap();
            Err(TokenError::CommonTokenError(
                i..i + ch.len_utf8(),
                CommonTokenError::CurrentErrors("EndsWithQuote"),
            ))
        }
    }
}
pub use tokens::*;

#[test]
fn language() {
    let v = Ctxt::from(
        r#"
    fn sdf<T = Type, T, const T: Type>(a: A = "a", b: C, c = "c") -> Type {
    }
    
    struct Name<T=Type, T, const T: Type> {
        sdfsdf: Type = "sdfsf"
        sdfsfd
    }
    "#,
    );
    let b = Items::recog(&v);
    b.map(|b| {
        dbg!("pass");
        dbg!(b.clone());
        dbg!(&v.code.source[dbg!(b.span().end)..]);
    })
    .map_err(|e| {
        dbg!("error");
        dbg!(e);
    });
}

mod tmp {
    use super::*;

    mod tmp1 {
        use super::*;

        peg_grammar! {
            Tmp1 ::= Tok1+ (Tok3 Tok1+)+
                Tok1 ::= "aa"
                Tok3 ::= ";"
            R ::= "a" / "a"
            A ::= "a" / "a" / ("a" / "a" "a" / "b")
        }

        #[test]
        fn test() {
            let s = r#"aa;aa;ca;aa;aa"#;
            let v = Ctxt::from(s);
            let b = Tmp1::recog(&v);

            b.map(|b| {
                dbg!("pass");
                dbg!(b.clone());
                dbg!(&v.code.source[dbg!(b.span().end)..]);
            })
            .map_err(|e| {
                dbg!("error");
                dbg!(e);
            });
        }
    }

    mod tmp4 {
        use super::*;

        peg_grammar! {
            Tmp4 ::= "bb"* "23"
        }
        #[test]
        fn test() {
            let s = r#"bbbcb23b23"#;
            let v = Ctxt::from(s);
            let b = Tmp4::recog(&v);

            b.map(|b| {
                dbg!("pass");
                dbg!(b.clone());
                dbg!(&v.code.source[dbg!(b.span().end)..]);
            })
            .map_err(|e| {
                dbg!("error");
                dbg!(e);
            });
        }
    }
}

mod recovery {
    use super::*;

    peg_grammar! {
        Tmp2 ::= Tmp3 "str"
            Tmp3 ::= (Tok / (!Tok !"str" ".")+)+
        Tmp1 ::= Tmp "fn"
            Tmp ::= (Tok / (!Tok !"fn" ".")+)+
        Tok ::= "aa;"
    }

    #[test]
    fn test1() {
        let s = r#"aa;aa;ca;aa;aa;fn"#;
        let v = Ctxt::from(s);
        let b = Tmp1::recog(&v);

        b.map(|b| {
            dbg!("pass");
            dbg!(b.clone());
            dbg!(&v.code.source[dbg!(b.span().end)..]);
        })
        .map_err(|e| {
            dbg!("error");
            dbg!(e);
        });
    }
}
