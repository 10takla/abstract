use super::{constructor::Struct, *};
use lexer3_macros::{peg_grammar, EnumRecog, RegularToken, Spanable};

peg_grammar! {
    Items ::= Item*
        Item ::= Fn
            Fn ::= "fn" Ident StructBody
                StructBody ::= r"\{" FunctionParameters r"\}"
                    FunctionParameters ::= FunctionParam ","?
                        FunctionParam ::= DefaultField / Field
                            DefaultField ::= Field "=" Value
                                @Field ::= (Ident ":" Type) / Ident

    Type ::= Ident
    Value ::= Ident / String

    @Ident ::= r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"
    String ::= r#""[^"\\]*(?:\\.[^"\\]*)*""#
}

#[test]
fn language() {
    let v = Ctxt::from(r#"fnsdf{sdf}"#);
    let b = dbg!(Items::recog(&v).unwrap());
    dbg!(&v.code.source[b.span().end..]);
}
