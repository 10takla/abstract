pub mod left_right;

use super::literal::{Literal, LiteralType};
use crate::lexer::{
    items::{item::ident::Ident, Code},
    Parse, Slicable, Slice,
};
use left_right::LeftRight;
use std::fmt::Display;
use std_reset::prelude::Deref;

#[derive(PartialEq, Debug, Clone, Deref, Hash, Eq)]
pub struct Assign(pub LeftRight<Ident, Literal>);

impl Assign {
    pub fn new(
        ident_slice: [usize; 2],
        (literal_type, literal_slice): (LiteralType, [usize; 2]),
        code: &Code,
    ) -> Self {
        Self(LeftRight {
            left: Ident::new(ident_slice, code),
            right: Literal::new(literal_type, literal_slice, code),
        })
    }
}

impl Parse for Assign {
    fn parse(code: &Code) -> Option<Self> {
        LeftRight::parse(code, |code| {
            let (i, char) = code.iter().next()?;
            (char == '=').then_some(i)
        })
        .map(Self)
    }
}

impl Slicable for Assign {
    fn get_start(&self) -> usize {
        self.left.get_start()
    }
    fn get_end(&self) -> usize {
        self.right.get_end()
    }
}

impl Display for Assign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssignAnd({} = {})", self.left, self.right)
    }
}

#[test]
fn parse_assign() {
    let check = |a, b: ([usize; 2], (LiteralType, [usize; 2]))| {
        let code = &mut Code::new(a);
        assert_eq!(
            Assign::parse(&mut Code::new(a)),
            Some(Assign(LeftRight {
                left: Ident::new(b.0, code),
                right: Literal {
                    type_: b.1 .0,
                    slice: Slice::new(b.1 .1, code)
                }
            }))
        );
    };
    let check_none = |a| {
        assert_eq!(Assign::parse(&mut Code::new(a)), None);
    };

    check(" abc = 6", ([1, 3], (LiteralType::Number, [7, 7])));
    check("abc=6", ([0, 2], (LiteralType::Number, [4, 4])));
    check(" abc=6 ", ([1, 3], (LiteralType::Number, [5, 5])));
    check(" abc = \"root\"", ([1, 3], (LiteralType::String, [7, 12])));

    check_none("abc =");
    check_none("abc = ");
    check_none("abc=");
}
