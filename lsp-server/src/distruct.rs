use super::SYMBOL;
use crate::BLOCK;
use parser::{language::*, parser::{CommonRecog, Spanable}};
use tower_lsp::lsp_types::SemanticTokenType;

type DistrIter = Vec<DistrItem>;
type DistrItem = ([usize; 2], SemanticTokenType);

// итератор для ленивого на всех вложенных уровнях
pub fn distruct_items(items: &Vec<Item>) -> impl Iterator<Item = DistrItem> {
    let mut vec = vec![];
    items.distruct(&mut vec);
    vec.into_iter()
}

trait DI {
    fn push_t(&mut self, value: &impl StartEnd, t: SemanticTokenType);
}
impl DI for DistrIter {
    fn push_t(&mut self, value: &impl StartEnd, t: SemanticTokenType) {
        Vec::push(self, (value.start_end(), t));
    }
}

pub trait Distruct {
    fn distruct(&self, vec: &mut DistrIter);
}

impl Distruct for Vec<Item> {
    fn distruct(&self, vec: &mut DistrIter) {
        // self.iter().for_each(|v| {
            // v.distruct(vec);
        // });
    }
}

// impl Distruct for Item {
//     fn distruct(&self, vec: &mut DistrIter) {
//         use Item::*;
//         match self {
//             FnC(v) => v.distruct(vec),
//             StructC(v) => v.distruct(vec),
//             TraitC(v) => v.distruct(vec),
//             ImplV(v) => v.distruct(vec),
//             AnyBlock(v) => v.distruct(vec),
//             ConstC(v) => v.distruct(vec),
//             AssignExpr(v) => v.distruct(vec),
//             Literal(v) => v.distruct(vec),
//             Ident(v) => v.distruct(vec),
//         }
//     }
// }

// mod fn_c {
//     use super::*;
//     use lexer::lexer2::FnHead;

//     impl Distruct for FnC {
//         fn distruct(&self, vec: &mut DistrIter) {
//             self.0.distruct(vec);
//             self.1.distruct(vec);
//         }
//     }

//     impl Distruct for FnHead {
//         fn distruct(&self, vec: &mut DistrIter) {
//             self.0.distruct(vec);
//             vec.push_t(&self.1, SemanticTokenType::FUNCTION);
//             use Args::*;
//             match &self.2 {
//                 StructArgsC(v) => {
//                     v.bracket(vec, SemanticTokenType::FUNCTION);
//                 }
//                 TupleType(v) => {
//                     v.bracket(vec, SemanticTokenType::FUNCTION);
//                 }
//             }
//         }
//     }
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

// macro_rules! fast {
//     (@symbls $($ident:ident)*) => {
//         $(
//             impl Distruct for $ident {
//                 fn distruct(&self, vec: &mut DistrIter) {
//                     vec.push_t(self, SYMBOL);
//                 }
//             }
//         )*
//     };
//     (@items $($ident:ident)*) => {
//         $(
//             impl Distruct for $ident {
//                 fn distruct(&self, vec: &mut DistrIter) {
//                     self.iter().for_each(|v| {
//                         v.distruct(vec);
//                     });
//                 }
//             }
//         )*
//     };
//     (@brakets $($ident:ident)*) => {
//         $(
//             impl Bracketable for $ident {
//                 fn bracket(&self, vec: &mut DistrIter, type_: SemanticTokenType) {
//                     vec.push_t(&self.0, type_.clone());
//                     self.1.distruct(vec);
//                     vec.push_t(&self.2, type_);
//                 }
//             }
//         )*
//     };
//     (@keywords $($ident:ident)*) => {
//         $(
//             impl Distruct for $ident {
//                 fn distruct(&self, vec: &mut DistrIter) {
//                     vec.push_t(self, SemanticTokenType::KEYWORD);
//                 }
//             }
//         )*
//     };
// }

// fast!(@items StructArgsI TupleTypeI);
// fast!(@symbls Eq OpEq Colon Comma);
// fast!(@brakets StructArgsC TupleType);
// fast!(@keywords Crate Fn Const Struct Trait Let Impl For);

// trait Bracketable {
//     fn bracket(&self, vec: &mut DistrIter, type_: SemanticTokenType);
// }

trait StartEnd: Spanable {
    fn start_end(&self) -> [usize; 2] {
        let v = self.span();
        [v.start, v.end]
    }
}

impl<T: Spanable> StartEnd for T {}
