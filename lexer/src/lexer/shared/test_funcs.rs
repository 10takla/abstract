use super::{
    code::Code,
    parse::{
        diag::{DiagParse, Diags},
        Parse,
    },
};
use crate::Diag;
use std::fmt::Debug;

pub fn check<'s, T: Parse<'s> + PartialEq + Debug>(
    source: &'s str,
    get_item: impl FnOnce(&Code<'s>) -> T,
) {
    let code = &mut Code::new(source);
    assert_eq!(
        T::parse(code, &mut Default::default(), &mut Default::default()),
        Some(get_item(code))
    );
}

pub fn check_none<'s, T>(source: &'s str)
where
    T: DiagParse<'s> + PartialEq + Debug,
{
    assert_eq!(
        T::parse(
            &mut Code::new(source),
            &mut Default::default(),
            &mut Default::default()
        ),
        None
    );
}

pub fn check_diag<'s, D, I: DiagParse<'s, Diag = D, Diags = Diags<D>>>(
    source: &'s str,
    diags: Vec<Diag<D>>,
) where
    I: PartialEq + Debug,
    D: PartialEq + Debug,
{
    assert_eq!(
        I::diag(&Code::new(source), &mut Default::default()),
        Err(Diags::from_errors(diags))
    );
}
