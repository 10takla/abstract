mod lexer;
use items::Items;
pub use lexer::*;

pub fn parse(source: &'static str) -> Items {
    Items::parse(&Code::new(source)).unwrap()
}
