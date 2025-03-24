use super::SYMBOL;
use crate::{BLOCK, ERROR};
use lsp_server_macros::distruct;
use parser::{
    language::*,
    parser::{AndPredicate, CommonRecog, Empty, ErrorRecovery, Opt, Spanable, Token},
    tuple_impl,
};
use std::ops::Range;
use tower_lsp::lsp_types::SemanticTokenType;

type DistrIter = Vec<DistrItem>;
pub type DistrItem = (Range<usize>, SemanticTokenType);

// итератор для ленивого на всех вложенных уровнях
pub fn distruct_items(items: &Vec<Item>) -> impl Iterator<Item = DistrItem> + std::fmt::Debug {
    let mut vec = vec![];
    items.distruct(&mut vec);
    vec.into_iter()
}

trait DI {
    fn push_t(&mut self, value: &impl Spanable, t: SemanticTokenType);
}
impl DI for DistrIter {
    fn push_t(&mut self, value: &impl Spanable, t: SemanticTokenType) {
        Vec::push(self, (value.span(), t));
    }
}

pub trait Distruct {
    fn distruct(&self, vec: &mut DistrIter);
}

impl<T> Distruct for Token<T> {
    default fn distruct(&self, _: &mut DistrIter) {}
}

impl Distruct for WhiteSpaces {
    fn distruct(&self, _: &mut DistrIter) {}
}

impl Distruct for I {
    fn distruct(&self, _: &mut DistrIter) {}
}

impl Distruct for Ident {
    fn distruct(&self, vec: &mut DistrIter) {
        vec.push_t(self, SemanticTokenType::VARIABLE);
    }
}

impl Distruct for String {
    fn distruct(&self, vec: &mut DistrIter) {
        vec.push_t(self, SemanticTokenType::STRING);
    }
}

impl<T: Distruct> Distruct for Vec<T> {
    fn distruct(&self, vec: &mut DistrIter) {
        self.into_iter().for_each(|v| {
            v.distruct(vec);
        });
    }
}

impl Distruct for Type {
    fn distruct(&self, _: &mut DistrIter) {}
}

impl<T: Distruct> Distruct for Box<T> {
    fn distruct(&self, vec: &mut DistrIter) {
        (**self).distruct(vec);
    }
}

distruct!(enum_ Enum0 2);
distruct!(enum_ Enum2 2);
distruct!(enum_ StructParam 2);
distruct!(enum_ Enum1 2);
distruct!(enum_ FunctionParam 2);
distruct!(enum_ GenericParam 2);
distruct!(enum_ Item 4);
distruct!(enum_ BlockContent 2);
distruct!(enum_ Value 3);

impl<T: CommonRecog<Output: Spanable>> Distruct for ErrorRecovery<T> {
    fn distruct(&self, vec: &mut DistrIter) {
        vec.push_t(self, ERROR);
    }
}

impl<T: Distruct> Distruct for Opt<T> {
    fn distruct(&self, vec: &mut DistrIter) {
        match self {
            Self::Some(v) => {
                v.distruct(vec);
            }
            Self::None(..) => {}
        };
    }
}

impl<T: CommonRecog<Output: Distruct>> Distruct for AndPredicate<T> {
    fn distruct(&self, vec: &mut DistrIter) {
        self.0.distruct(vec);
    }
}

impl Distruct for Empty {
    fn distruct(&self, _: &mut DistrIter) {}
}

distruct!(struct_ 18);
// impl<T> Distruct for T {
//     default fn distruct(&self, _: &mut DistrIter) {}
// }

// mod fn_ {
//     use parser::tuple_impl;
//     use super::*;

//     macro_rules! d {
//         ($($a:ident)+) => {
//             impl<$($a: Distruct),+> Distruct for ($($a),+)
//                 fn distruct(&self, vec: &mut DistrIter) {
//                     distruct!(struct_ 4);
//                 }
//             }
//         };
//     }

//     tuple_impl!(d);
// }

// mod trait_ {
//     use super::*;
//     use crate::TRAIT;
//     use lexer::lexer2::{MethodsC, MethodsI};
//     impl Distruct for TraitC {
//         fn distruct(&self, vec: &mut DistrIter) {
//             self.0.distruct(vec);
//             vec.push_t(&self.1, SemanticTokenType::VARIABLE);
//             self.2.distruct(vec);
//         }
//     }
//     impl Distruct for MethodsC {
//         fn distruct(&self, vec: &mut DistrIter) {
//             vec.push_t(&self.0, TRAIT);
//             self.1.distruct(vec);
//             vec.push_t(&self.2, TRAIT);
//         }
//     }
//     impl Distruct for MethodsI {
//         fn distruct(&self, vec: &mut DistrIter) {
//             self.iter().for_each(|v| {
//                 v.distruct(vec);
//             });
//         }
//     }
// }
// mod impl_ {
//     use super::*;
//     use crate::IMPL;
//     use lexer::lexer2::{ConstC, ImplItemsC, ImplItemsV, ImplV};

//     impl Distruct for ImplV {
//         fn distruct(&self, vec: &mut DistrIter) {
//             use ImplV::*;
//             match self {
//                 ImplFor(v) => {
//                     v.0.distruct(vec);
//                     vec.push_t(&v.1, SemanticTokenType::VARIABLE);
//                     v.2.distruct(vec);
//                     v.3.distruct(vec);
//                     v.4.distruct(vec);
//                 }
//                 ImplC(v) => {
//                     v.0.distruct(vec);
//                     v.1.distruct(vec);
//                     v.2.distruct(vec);
//                 }
//             }
//         }
//     }
//     impl Distruct for ImplItemsC {
//         fn distruct(&self, vec: &mut DistrIter) {
//             vec.push_t(&self.0, IMPL);
//             self.1.iter().for_each(|v: &lexer::lexer2::ImplItemsV| {
//                 v.distruct(vec);
//             });
//             vec.push_t(&self.2, IMPL);
//         }
//     }
//     impl Distruct for ImplItemsV {
//         fn distruct(&self, vec: &mut DistrIter) {
//             use ImplItemsV::*;
//             match self {
//                 ConstC(v) => {
//                     v.distruct(vec);
//                 }
//                 FnC(v) => {
//                     v.distruct(vec);
//                 }
//             }
//         }
//     }
//     impl Distruct for ConstC {
//         fn distruct(&self, vec: &mut DistrIter) {
//             self.0.distruct(vec);
//             self.1.distruct(vec);
//         }
//     }
// }

// impl Distruct for StructC {
//     fn distruct(&self, vec: &mut DistrIter) {
//         self.0.distruct(vec);
//         vec.push_t(&self.1, SemanticTokenType::VARIABLE);
//         use Args::*;
//         match &self.2 {
//             StructArgsC(v) => {
//                 v.bracket(vec, SemanticTokenType::STRUCT);
//             }
//             TupleType(v) => {
//                 v.bracket(vec, SemanticTokenType::STRUCT);
//             }
//         }
//     }
// }

// impl Distruct for IdentAndTypeC {
//     fn distruct(&self, vec: &mut DistrIter) {
//         self.0.distruct(vec);
//         self.1.distruct(vec);
//         self.2.distruct(vec);
//     }
// }

// mod block {
//     use super::*;
//     use lexer::lexer2::BlockItems;
//     impl Distruct for AnyBlock {
//         fn distruct(&self, vec: &mut DistrIter) {
//             use self::AnyBlock::*;
//             match self {
//                 Block(v) => v.distruct(vec),
//                 NamedBlock(v) => v.distruct(vec),
//                 NamedDistrBlock(v) => {
//                     v.0.distruct(vec);
//                     vec.push_t(&v.1, BLOCK);
//                 }
//                 DistrBlock(v) => {
//                     vec.push_t(v, BLOCK);
//                 }
//             }
//         }
//     }
//     impl Distruct for NamedBlock {
//         fn distruct(&self, vec: &mut DistrIter) {
//             vec.push_t(&self.0, BLOCK);
//             self.1.distruct(vec);
//         }
//     }
//     impl Distruct for Block {
//         fn distruct(&self, vec: &mut DistrIter) {
//             vec.push_t(&self.0, BLOCK);
//             self.1.distruct(vec);
//             vec.push_t(&self.2, BLOCK);
//         }
//     }
//     impl Distruct for BlockItems {
//         fn distruct(&self, vec: &mut DistrIter) {
//             self.iter().for_each(|v| {
//                 v.distruct(vec);
//             });
//         }
//     }
// }
// mod assign_expr {
//     use super::*;
//     use tokio::task::Id;
//     impl Distruct for AssignExpr {
//         fn distruct(&self, vec: &mut DistrIter) {
//             use self::AssignExpr::*;
//             match self {
//                 Assign(v) => {
//                     v.distruct(vec);
//                 }
//                 AssignAnd(v) => {
//                     v.0.distruct(vec);
//                     v.1.distruct(vec);
//                     v.2.distruct(vec);
//                 }
//             }
//         }
//     }
//     impl Distruct for Assign {
//         fn distruct(&self, vec: &mut DistrIter) {
//             self.0.distruct(vec);
//             self.1.distruct(vec);
//             self.2.distruct(vec);
//         }
//     }
//     impl Distruct for IdentAndType {
//         fn distruct(&self, vec: &mut DistrIter) {
//             use IdentAndType::*;
//             match self {
//                 IdentAndTypeC(v) => v.distruct(vec),
//                 Ident(v) => v.distruct(vec),
//             }
//         }
//     }
// }
// impl Distruct for Literal {
//     fn distruct(&self, vec: &mut DistrIter) {
//         use self::Literal::*;
//         match self {
//             String(..) => vec.push_t(self, SemanticTokenType::STRING),
//             Number(..) => vec.push_t(self, SemanticTokenType::NUMBER),
//         }
//     }
// }

// mod type_ {
//     use super::{DistrIter, Distruct, DI};
//     use lexer::lexer2::{TupleType, Type};
//     use tower_lsp::lsp_types::SemanticTokenType;

//     impl Distruct for Type {
//         fn distruct(&self, vec: &mut DistrIter) {
//             vec.push_t(self, SemanticTokenType::TYPE);
//         }
//     }
// }

// impl Distruct for Ident {
//     fn distruct(&self, vec: &mut DistrIter) {
//         vec.push_t(self, SemanticTokenType::VARIABLE);
//     }
// }

// impl Distruct for Ignore {
//     fn distruct(&self, _: &mut DistrIter) {}
// }

macro_rules! fast {
    // (@symbls $($ident:ident)*) => {
    //     $(
    //         impl Distruct for $ident {
    //             fn distruct(&self, vec: &mut DistrIter) {
    //                 vec.push_t(self, SYMBOL);
    //             }
    //         }
    //     )*
    // };
    // (@items $($ident:ident)*) => {
    //     $(
    //         impl Distruct for $ident {
    //             fn distruct(&self, vec: &mut DistrIter) {
    //                 self.iter().for_each(|v| {
    //                     v.distruct(vec);
    //                 });
    //             }
    //         }
    //     )*
    // };
    // (@brakets $($ident:ident)*) => {
    //     $(
    //         impl Bracketable for $ident {
    //             fn bracket(&self, vec: &mut DistrIter, type_: SemanticTokenType) {
    //                 vec.push_t(&self.0, type_.clone());
    //                 self.1.distruct(vec);
    //                 vec.push_t(&self.2, type_);
    //             }
    //         }
    //     )*
    // };
    (@keywords $($ident:ident)*) => {
        $(
            impl Distruct for $ident {
                fn distruct(&self, vec: &mut DistrIter) {
                    vec.push_t(self, SemanticTokenType::KEYWORD);
                }
            }
        )*
    };
}

// fast!(@items StructArgsI TupleTypeI);
// fast!(@symbls Eq OpEq Colon Comma);
// fast!(@brakets StructArgsC TupleType);
fast!(@keywords FnKeyword ConstKeyword StructKeyword Let);

// trait Bracketable {
//     fn bracket(&self, vec: &mut DistrIter, type_: SemanticTokenType);
// }
