use super::{
    assign::{left_right::LeftRight, Assign},
    literal::LiteralType,
};
use crate::lexer::{
    check, check_none,
    items::{Code, Slicable},
    Parse,
};
use std::fmt::Display;

#[derive(PartialEq, Debug, Clone)]
pub struct AssignAnd {
    pub type_: AssignAndType,
    pub val: Assign,
}

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub enum AssignAndType {
    Add,
    Sub,
    Mul,
    Div,
}

impl AssignAnd {
    pub fn new(
        type_: AssignAndType,
        (slice, (literal_type, literal_slice)): ([usize; 2], (LiteralType, [usize; 2])),
        code: &Code,
    ) -> Self {
        Self {
            type_,
            val: Assign::new(slice, (literal_type, literal_slice), code),
        }
    }
}

impl Parse for AssignAnd {
    fn parse(code: &Code) -> Option<Self> {
        let mut assign_type = None;

        LeftRight::parse(code, |code| {
            let mut iter = code.iter();

            let (_, char) = iter.next()?;
            match char {
                '+' => assign_type = Some(AssignAndType::Add),
                '-' => assign_type = Some(AssignAndType::Sub),
                '*' => assign_type = Some(AssignAndType::Mul),
                '/' => assign_type = Some(AssignAndType::Div),
                _ => return None,
            }

            let (i, char) = iter.next()?;
            (char == '=').then_some(i)
        })
        .map(|lr| Self {
            type_: assign_type.unwrap(),
            val: Assign(lr),
        })
    }
}

impl Slicable for AssignAnd {
    fn get_start(&self) -> usize {
        self.val.get_start()
    }
    fn get_end(&self) -> usize {
        self.val.get_end()
    }
}

impl Display for AssignAnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AssignAnd({} {} {})",
            self.val.left, self.type_, self.val.right,
        )
    }
}

impl Display for AssignAndType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AssignAndType::Add => "Add(+=)",
                AssignAndType::Sub => "Sub(-=)",
                AssignAndType::Mul => "Mul(*=)",
                AssignAndType::Div => "Div(/=)",
            },
        )
    }
}

#[test]
fn parse_assign_and() {
    check(" abc += 6", |code| {
        AssignAnd::new(
            AssignAndType::Add,
            ([1, 3], (LiteralType::Number, [8, 8])),
            code,
        )
    });
    check(" abc -= 6", |code| {
        AssignAnd::new(
            AssignAndType::Sub,
            ([1, 3], (LiteralType::Number, [8, 8])),
            code,
        )
    });
    check(" abc *= 6", |code| {
        AssignAnd::new(
            AssignAndType::Mul,
            ([1, 3], (LiteralType::Number, [8, 8])),
            code,
        )
    });
    check(" abc /= 6", |code| {
        AssignAnd::new(
            AssignAndType::Div,
            ([1, 3], (LiteralType::Number, [8, 8])),
            code,
        )
    });

    check("abc+=6", |code| {
        AssignAnd::new(
            AssignAndType::Add,
            ([0, 2], (LiteralType::Number, [5, 5])),
            code,
        )
    });

    // error
    check_none::<AssignAnd>("a + ");
    check_none::<AssignAnd>("");
    check_none::<AssignAnd>(" ");
    check_none::<AssignAnd>("   ");
}
