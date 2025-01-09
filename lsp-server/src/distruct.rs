use crate::BLOCK;
use lexer::lexer2::{
    AnyBlock, Args, AssignExpr, Block, FnC, Ident, Idents, Ignore, ImplC, Item, Items, Keyword,
    Literal, NamedBlock, Slicable, StructC, TraitC,
};
use std::iter::{empty, once};
use tower_lsp::lsp_types::SemanticTokenType;

// итератор для ленивого на всех вложенных уровнях
type DistrIter = Box<dyn Iterator<Item = DistrItem>>;
type DistrItem = ([usize; 2], SemanticTokenType);

// итератор для ленивого прохода
pub fn distruct_items<'a>(items: &'a Items) -> impl Iterator<Item = DistrItem> + 'a {
    items.iter().flat_map(Distruct::distruct)
}

fn fast_once<'a>(v: &impl StartEnd, t: SemanticTokenType) -> impl Iterator<Item = DistrItem> + 'a {
    once((v.start_end(), t))
}

fn fast_box<'a>(v: &impl StartEnd, t: SemanticTokenType) -> DistrIter {
    Box::new(fast_once(v, t))
}

pub trait Distruct {
    fn distruct(&self) -> DistrIter {
        Box::new(empty())
    }
}

impl Distruct for Item {
    fn distruct(&self) -> DistrIter {
        use Item::*;
        match self {
            FnC(v) => v.distruct(),
            StructC(v) => v.distruct(),
            TraitC(v) => v.distruct(),
            ImplC(v) => v.distruct(),
            AnyBlock(v) => v.distruct(),
            AssignExpr(v) => v.distruct(),
            Literal(v) => v.distruct(),
            Idents(v) => v.distruct(),
            Ignore(v) => v.distruct(),
        }
    }
}

mod fn_c {
    use super::*;
    use lexer::lexer2::FnHead;

    impl Distruct for FnC {
        fn distruct(&self) -> DistrIter {
            Box::new(self.0.distruct().chain(self.1.distruct()))
        }
    }

    impl Distruct for FnHead {
        // fn distruct(&self) -> DistrIter {
        //     Box::new(
        //         fast_box(&self.0, SemanticTokenType::KEYWORD)
        //             .chain(fast_box(&self.1, SemanticTokenType::STRUCT))
        //             .chain(self.2.color()),
        //     )
        // }
    }
}

impl Distruct for StructC {
    fn distruct(&self) -> DistrIter {
        Box::new(
            fast_box(&self.0, SemanticTokenType::KEYWORD)
                .chain(fast_box(&self.1, SemanticTokenType::FUNCTION))
                // .chain(Box::new(self.2.color())),
        )
    }
}
impl Distruct for TraitC {}
impl Distruct for ImplC {}
impl Distruct for AnyBlock {
    fn distruct(&self) -> DistrIter {
        use self::AnyBlock::*;
        match self {
            Block(v) => v.distruct(),
            NamedBlock(v) => v.distruct(),
            NamedDistrBlock(v) => Box::new(v.0.distruct().chain(fast_once(&v.1, BLOCK))),
            DistrBlock(v) => fast_box(v, BLOCK),
        }
    }
}
impl Distruct for NamedBlock {
    fn distruct(&self) -> DistrIter {
        Box::new(fast_once(&self.0, BLOCK).chain(self.1.distruct()))
    }
}

impl Distruct for Block {
    fn distruct(&self) -> DistrIter {
        Box::new(
            fast_once(&self.0, BLOCK)
                // .chain(distruct_items(&self.1))
                .chain(fast_once(&self.2, BLOCK)),
        )
    }
}
impl Distruct for AssignExpr {
    fn distruct(&self) -> DistrIter {
        use self::AssignExpr::*;
        match self {
            Assign(v) => Box::new(v.0.distruct().chain(v.2.distruct())),
            AssignAnd(v) => Box::new(v.0.distruct().chain(v.2.distruct())),
        }
    }
}
impl Distruct for Literal {
    fn distruct(&self) -> DistrIter {
        use self::Literal::*;
        match self {
            String(..) => fast_box(self, SemanticTokenType::STRING),
            Number(..) => fast_box(self, SemanticTokenType::NUMBER),
        }
    }
}
impl Distruct for Idents {
    fn distruct(&self) -> DistrIter {
        use self::Idents::*;
        match self {
            Ident(v) => v.distruct(),
            Keyword(v) => fast_box(v, SemanticTokenType::KEYWORD),
        }
    }
}
impl Distruct for Ident {
    fn distruct(&self) -> DistrIter {
        fast_box(self, SemanticTokenType::VARIABLE)
    }
}

impl Distruct for Keyword {
    fn distruct(&self) -> DistrIter {
        fast_box(self, SemanticTokenType::KEYWORD)
    }
}

impl Distruct for Ignore {}

trait Colorize {
    fn color(&self) -> impl Iterator<Item = DistrItem> {
        self.cl().into_iter().map(|(a, b)| (a.start_end(), b))
    }
    fn cl(&self) -> Vec<(&dyn StartEnd, SemanticTokenType)>;
}

impl Colorize for Args {
    fn cl(&self) -> Vec<(&dyn StartEnd, SemanticTokenType)> {
        use Args::*;
        match self {
            StructArgsC(v) => {
                vec![
                    (&v.0, SemanticTokenType::FUNCTION),
                    (&v.2, SemanticTokenType::FUNCTION),
                ]
            }
            TupleArgsC(v) => {
                vec![
                    (&v.0, SemanticTokenType::FUNCTION),
                    (&v.2, SemanticTokenType::FUNCTION),
                ]
            }
        }
    }
}

trait StartEnd: Slicable {
    fn start_end(&self) -> [usize; 2] {
        let v = self.slice();
        [*v.start(), *v.end()]
    }
}

impl<T: Slicable> StartEnd for T {}
