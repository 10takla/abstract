/// Лексер парсит конструкции языка из строки кода через отлеживатель [`Cursor`].
/// [`Cursor`] следит за исходной строкой кода, предотвращая ее клонировние с помощью единного времени жизни:
/// 's - время жизни исходной строки (source)
mod lexer;
use items::Items;
pub use lexer::*;

pub fn parse(source: &str) -> Items<'_> {
    Items::parse(&Code::new(source)).unwrap()
}
