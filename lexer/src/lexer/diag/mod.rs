use super::{items::shared::whitespaces::Whitespaces, Code, Parse, Slice, IGNORE};
use macros::Slicable;

#[derive(PartialEq, Debug)]
enum Item<'s> {
    Number(Number<'s>),
    String(String<'s>),
}

type Diags = Vec<(usize, std::string::String)>;

impl<'s> Item<'s> {
    fn diag(code: &Code<'s>) -> (Option<Self>, Diags) {
        let mut diags = vec![];
        let t = [
            Box::new(|| {
                let (v, d) = Number::diag(code);
                (v.map(|v| Self::Number(v)), d)
            }) as Box<dyn Fn() -> (Option<Self>, Diags)>,
            Box::new(|| {
                let (v, d) = String::diag(code);
                (v.map(|v| Self::String(v)), d)
            }),
        ]
        .into_iter()
        .find_map(|f| {
            let (v, d) = f();
            diags.extend(d);
            v
        });
        (t, diags)
    }
}

#[test]
fn diag_test() {
    let check = |source| {
        assert_eq!(Item::diag(&Code::new(source)), (None, vec![]));
    };

    check("43c");
}

#[derive(PartialEq, Debug, Slicable)]
pub struct Number<'s>(pub Slice<'s>);

impl<'s> Number<'s> {
    fn diag(code: &Code<'s>) -> (Option<Self>, Diags) {
        let mut diags = vec![];
        (Self::parse(code, &mut diags), diags)
    }
    fn parse(code: &Code<'s>, diags: &mut Diags) -> Option<Self> {
        let code = &mut code.clone();

        Whitespaces::parse_and_consume(code);
        let mut iter = code.iter();
 
        let (i, char) = iter.next().or_else(|| {
            diags.push((code.cursor, "Должно начинатся с числа".into()));
            None
        })?;
        let start = if char.is_digit(10) {
            if i == code.len() - 1 {
                return Some(Self(Slice::new(i..=i, code)));
            }
            i
        } else {
            return None;
        };

        let t = || {
            for (i, char) in iter {
                if IGNORE.contains(&char) {
                    return Some(i - 1);
                }
                if char.is_digit(10) {
                    if i == code.len() - 1 {
                        return Some(i);
                    }
                    continue;
                }
                return None;
            }
            None
        };
        let end = t()?;

        Some(Self(Slice::new(start..=end, code)))
    }
}

#[derive(Debug, PartialEq, Slicable)]
pub struct String<'s>(pub Slice<'s>);

impl<'s> String<'s> {
    fn diag(code: &Code<'s>) -> (Option<Self>, Diags) {
        let mut diags = vec![];
        (Self::parse(code, &mut diags), diags)
    }
    fn parse(code: &Code<'s>, diags: &mut Diags) -> Option<Self> {
        let code = &mut code.clone();

        Whitespaces::parse_and_consume(code);
        let mut iter = code.iter();

        let (i, char) = iter.next()?;
        let start = (char == '"').then_some(i)?;

        for (i, char) in iter {
            if char == '"' {
                return Some(Self(Slice::new(start..=i, code)));
            }
        }
        None
    }
}
