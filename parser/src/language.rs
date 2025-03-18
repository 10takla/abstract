use crate::{parser::*, Ctxt, *};
use macros::peg_grammar;

peg_grammar! {
    Items2 ::= I (Item I)*
    Items ::= (Item / ItemError)*
        ItemError ::= (!Item ".")+
        Item ::= Fn / Struct_ / Ident / WhiteSpaces
            Fn ::= "fn" I Ident I (GenericParams I)? r"\(" I (FunctionParameters I)? r"\)" (I FunctionReturn)? I FunctionBody
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
                StructFields ::= StructParam (I "," I StructParam)* (I ",")?
                    StructParam ::= DefaultStructField / StructField
                        DefaultStructField ::= StructField I "=" I Value
                            @StructField ::= (Ident I ":" I Type) / Type
    Type ::= Ident
    Value ::= Ident / String

    @Ident ::= TmpIdent
    // r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"
    String ::= r#""[^"\\]*(?:\\.[^"\\]*)*""#

    I ::= r"[\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u200E\u200F\u2028\u2029]*"
        WhiteSpaces ::= r"[\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u200E\u200F\u2028\u2029]+"

}

type TmpIdent = crate::parser::constructor::Ident;

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
