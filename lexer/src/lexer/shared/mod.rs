pub mod code;
pub mod parse;
pub mod slice;
pub mod test_funcs;

use code::Code;
use colored::Colorize;
use std::fmt::Display;

pub const IGNORE: [char; 3] = [' ', '\n', '\t'];

pub trait Diagn {
    const NAME: &'static str;
    fn display(&self, code: &Code, pos: usize) -> String {
        format!(
            "\"{}\". {}",
            code.get_char(pos).to_string().underline(),
            self.for_construct(code, pos),
        )
    }
    fn for_construct(&self, code: &Code, pos: usize) -> String {
        format!(
            "Должно {} для конструкции {}",
            self.expect(code, pos),
            Self::NAME
        )
    }
    fn expect(&self, code: &Code, pos: usize) -> impl Display;
}
