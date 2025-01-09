use super::SYMBOL;
use crate::BLOCK;
use lexer::lexer2::{
    AnyBlock, Args, AssignExpr, Block, FnC, Ident, Idents, Ignore, ImplC, Item, Items, Keyword,
    Literal, NamedBlock, Slicable, StructC, TraitC, *,
};
use tower_lsp::lsp_types::SemanticTokenType;

type DistrIter = Vec<DistrItem>;
type DistrItem = ([usize; 2], SemanticTokenType);

// итератор для ленивого на всех вложенных уровнях
pub fn distruct_items(items: &Items) -> impl Iterator<Item = DistrItem> {
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

impl Distruct for Items {
    fn distruct(&self, vec: &mut DistrIter) {
        self.iter().for_each(|v| {
            v.distruct(vec);
        });
    }
}

impl Distruct for Item {
    fn distruct(&self, vec: &mut DistrIter) {
        use Item::*;
        match self {
            FnC(v) => v.distruct(vec),
            StructC(v) => v.distruct(vec),
            TraitC(v) => v.distruct(vec),
            ImplV(v) => v.distruct(vec),
            AnyBlock(v) => v.distruct(vec),
            AssignExpr(v) => v.distruct(vec),
            Literal(v) => v.distruct(vec),
            Idents(v) => v.distruct(vec),
        }
    }
}

mod fn_c {
    use super::*;
    use lexer::lexer2::FnHead;

    impl Distruct for FnC {
        fn distruct(&self, vec: &mut DistrIter) {
            self.0.distruct(vec);
            self.1.distruct(vec);
        }
    }

    impl Distruct for FnHead {
        fn distruct(&self, vec: &mut DistrIter) {
            vec.push_t(&self.0, SemanticTokenType::KEYWORD);
            vec.push_t(&self.1, SemanticTokenType::FUNCTION);
            self.2.distruct(vec);
        }
    }
}

mod trait_ {
    use super::*;
    use crate::TRAIT;
    use lexer::lexer2::{MethodsC, MethodsI};
    impl Distruct for TraitC {
        fn distruct(&self, vec: &mut DistrIter) {
            vec.push_t(&self.0, SemanticTokenType::KEYWORD);
            vec.push_t(&self.1, SemanticTokenType::VARIABLE);
            self.2.distruct(vec);
        }
    }
    impl Distruct for MethodsC {
        fn distruct(&self, vec: &mut DistrIter) {
            vec.push_t(&self.0, TRAIT);
            self.1.distruct(vec);
            vec.push_t(&self.2, TRAIT);
        }
    }
    impl Distruct for MethodsI {
        fn distruct(&self, vec: &mut DistrIter) {
            self.iter().for_each(|v| {
                v.distruct(vec);
            });
        }
    }
}
mod impl_ {
    use super::*;
    use crate::{distruct, IMPL};
    use lexer::lexer2::{ConstC, ImplItemsC, ImplItemsI, ImplItemsV, ImplV};
    use tower_lsp::lsp_types::SemanticTokens;

    impl Distruct for ImplV {
        fn distruct(&self, vec: &mut DistrIter) {
            use ImplV::*;
            match self {
                ImplFor(v) => {
                    vec.push_t(&v.0, SemanticTokenType::KEYWORD);
                    vec.push_t(&v.1, SemanticTokenType::VARIABLE);
                    vec.push_t(&v.2, SemanticTokenType::KEYWORD);
                    v.3.distruct(vec);
                    v.4.distruct(vec);
                }
                ImplC(v) => {
                    vec.push_t(&v.0, SemanticTokenType::KEYWORD);
                    v.1.distruct(vec);
                    v.2.distruct(vec);
                }
            }
        }
    }
    impl Distruct for ImplItemsC {
        fn distruct(&self, vec: &mut DistrIter) {
            vec.push_t(&self.0, IMPL);
            self.1.iter().for_each(|v: &lexer::lexer2::ImplItemsV| {
                v.distruct(vec);
            });
            vec.push_t(&self.2, IMPL);
        }
    }
    impl Distruct for ImplItemsV {
        fn distruct(&self, vec: &mut DistrIter) {
            use ImplItemsV::*;
            match self {
                ConstC(v) => {
                    v.distruct(vec);
                }
                FnC(v) => {
                    v.distruct(vec);
                }
            }
        }
    }
    impl Distruct for ConstC {
        fn distruct(&self, vec: &mut DistrIter) {
            vec.push_t(&self.0, SemanticTokenType::KEYWORD);
            self.1.distruct(vec);
            self.2.distruct(vec);
            self.3.distruct(vec);
        }
    }
}

impl Distruct for StructC {
    fn distruct(&self, vec: &mut DistrIter) {
        vec.push_t(&self.0, SemanticTokenType::KEYWORD);
        vec.push_t(&self.1, SemanticTokenType::VARIABLE);
        use Args::*;
        match &self.2 {
            StructArgsC(v) => {
                vec.push_t(&v.0, SemanticTokenType::STRUCT);
                v.1.iter().for_each(|v| {
                    v.0.distruct(vec);
                    v.1.distruct(vec);
                    v.2.distruct(vec);
                    v.3.distruct(vec);
                });
                vec.push_t(&v.2, SemanticTokenType::STRUCT);
            }
            TupleArgsC(v) => {
                vec.push_t(&v.0, SemanticTokenType::STRUCT);
                v.1.iter().for_each(|v| {
                    v.0.distruct(vec);
                    v.1.distruct(vec);
                });
                vec.push_t(&v.2, SemanticTokenType::STRUCT);
            }
        }
    }
}

impl Distruct for AnyBlock {
    fn distruct(&self, vec: &mut DistrIter) {
        use self::AnyBlock::*;
        match self {
            Block(v) => v.distruct(vec),
            NamedBlock(v) => v.distruct(vec),
            NamedDistrBlock(v) => {
                v.0.distruct(vec);
                vec.push_t(&v.1, BLOCK);
            }
            DistrBlock(v) => {
                vec.push_t(v, BLOCK);
            }
        }
    }
}
impl Distruct for NamedBlock {
    fn distruct(&self, vec: &mut DistrIter) {
        vec.push_t(&self.0, BLOCK);
        self.1.distruct(vec);
    }
}
mod block {
    use super::*;
    use lexer::lexer2::BlockItems;

    impl Distruct for Block {
        fn distruct(&self, vec: &mut DistrIter) {
            vec.push_t(&self.0, BLOCK);
            self.1.distruct(vec);
            vec.push_t(&self.2, BLOCK);
        }
    }
    impl Distruct for BlockItems {
        fn distruct(&self, vec: &mut DistrIter) {
            self.iter().for_each(|v| {
                v.distruct(vec);
            });
        }
    }
}
impl Distruct for AssignExpr {
    fn distruct(&self, vec: &mut DistrIter) {
        use self::AssignExpr::*;
        match self {
            Assign(v) => {
                v.0.distruct(vec);
                v.1.distruct(vec);
                v.2.distruct(vec);
            }
            AssignAnd(v) => {
                v.0.distruct(vec);
                v.1.distruct(vec);
                v.2.distruct(vec);
            }
        }
    }
}
impl Distruct for Literal {
    fn distruct(&self, vec: &mut DistrIter) {
        use self::Literal::*;
        match self {
            String(..) => vec.push_t(self, SemanticTokenType::STRING),
            Number(..) => vec.push_t(self, SemanticTokenType::NUMBER),
        }
    }
}
impl Distruct for Idents {
    fn distruct(&self, vec: &mut DistrIter) {
        use self::Idents::*;
        match self {
            Ident(v) => v.distruct(vec),
            Keyword(v) => vec.push_t(v, SemanticTokenType::KEYWORD),
        }
    }
}
impl Distruct for Ident {
    fn distruct(&self, vec: &mut DistrIter) {
        vec.push_t(self, SemanticTokenType::VARIABLE);
    }
}

impl Distruct for Keyword {
    fn distruct(&self, vec: &mut DistrIter) {
        vec.push_t(self, SemanticTokenType::KEYWORD);
    }
}

impl Distruct for Ignore {
    fn distruct(&self, _: &mut DistrIter) {}
}

impl Distruct for Args {
    fn distruct(&self, vec: &mut DistrIter) {
        use Args::*;
        match self {
            StructArgsC(v) => {
                vec.push_t(&v.0, SemanticTokenType::FUNCTION);
                vec.push_t(&v.2, SemanticTokenType::FUNCTION);
            }
            TupleArgsC(v) => {
                vec.push_t(&v.0, SemanticTokenType::FUNCTION);
                vec.push_t(&v.2, SemanticTokenType::FUNCTION);
            }
        }
    }
}

macro_rules! symbls {
    ($($ident:ident)*) => {
        $(
            impl Distruct for $ident {
                fn distruct(&self, vec: &mut DistrIter) {
                    vec.push_t(self, SYMBOL);
                }
            }
        )*
    };
}

symbls!(Eq OpEq Colon Comma);

trait StartEnd: Slicable {
    fn start_end(&self) -> [usize; 2] {
        let v = self.slice();
        [*v.start(), *v.end()]
    }
}

impl<T: Slicable> StartEnd for T {}
