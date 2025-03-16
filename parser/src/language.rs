use crate::{parser::*, Ctxt, *};
use macros::peg_grammar;

peg_grammar! {
    Items2 ::= I (Item I)*
    Items ::= Item*
        Item ::= Fn / Struct_ / Ident / WhiteSpace+
            Fn ::= "fn" I Ident I (GenericParams I)? r"\(" I (FunctionParameters I)? r"\)" (I FunctionReturn)? I FunctionBody
                GenericParams2 ::= GenericParams*
                GenericParams ::= r"<" I GenericParamsBody? I r">"
                    GenericParamsBody ::= GenericParam (I "," I GenericParam)* (I ",")?
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
                StructFields ::= StructParam (I "," I StructParam)[* (I ",")] (I ",")?
                    StructParam ::= DefaultStructField / StructField
                        DefaultStructField ::= StructField I "=" I Value
                            @StructField ::= (Ident I ":" I Type) / Type
    Type ::= Ident
    Value ::= Ident / String

    @Ident ::= TmpIdent
    // r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"
    String ::= r#""[^"\\]*(?:\\.[^"\\]*)*""#

    I ::= r"[\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u200E\u200F\u2028\u2029]*"
        WhiteSpace ::= r"[\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u200E\u200F\u2028\u2029]"

}

type TmpIdent = crate::parser::constructor::Ident;

#[test]
fn language() {
    let v = Ctxt::from(
        r#"
    fn sdf<T = Type, T, const T: Type>(a: A = "a", b: C, c = "c") -> Type {
    }
    struct Name<T=Type, T, const T: Type> {
        sdfsdf: Type = "sdfsf",
        sdfsfd
    }
    "#,
    );
    let v = Ctxt::from(
        r#"
    fn sdf<T = Type, T, const T: Type>(a: A = "a", b: C, c = "c") -> Type {
    }
    "#,
    );
    let v = Ctxt::from(
        r#"<T = Type, y T, const T: Type><T = Type, T, const T: Type>"#,
    );
    let b = GenericParams2::recog(&v);
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

    peg_grammar! {
        Tmp4 ::= ("a" "bb" "c")[* "23"] "23"
        Tmp ::= Tok1[+ Tok3] (Tok3 Tok1[+ Tok3])+
            Tok1 ::= "a"
            Tok3 ::= ";"
    }

    #[test]
    fn tmp4() {
        let s = r#"abbcabcabbc23"#;
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

    #[test]
    fn tmp() {
        let s = r#"aa;ca;aa;aa;aa"#;
        let v = Ctxt::from(s);
        let b = Tmp::recog(&v);

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
