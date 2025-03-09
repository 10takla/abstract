use crate::{parser::*, Ctxt, *};
use macros::peg_grammar;

peg_grammar! {
    Items ::= Item*
        Item ::= Fn / Struct_ / I
            Fn ::= "fn" I Ident I (GenericParams I)? r"\(" I (FunctionParameters I)? r"\)" (I FunctionReturn)? I FunctionBody
                GenericParams ::= r"<" I GenericParamsBody? I r">"
                    GenericParamsBody ::= GenericParam (I "," I GenericParam)* (I ",")?
                        GenericParam ::= ConstParam / TypeParam
                            ConstParam ::= "const" I Ident I ":" I Type
                            TypeParam ::= Ident (I "=" I Type )?
                FunctionParameters ::= FunctionParam (I "," I FunctionParam)* (I ",")?
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

    @Ident ::= r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"
    String ::= r#""[^"\\]*(?:\\.[^"\\]*)*""#

    I ::= r"[\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u200E\u200F\u2028\u2029]*"
}

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
    let b = dbg!(Items::recog(&v).unwrap());
    dbg!(&v.code.source[b.span().end..]);
}
