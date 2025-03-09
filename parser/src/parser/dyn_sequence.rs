use super::*;
use crate::tuple_impl;
use std::process::Output;

type DynType = Box<dyn DynSpanable>;

/// DynSpanable для того чтобы для Spanable был Any, а Any в свою очередь для того чтобы был downcast
pub trait DynSpanable: Spanable + Any {}

impl dyn DynSpanable {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

impl<T: Spanable + Any> DynSpanable for T {}

pub trait DynSeqRecog {
    type Output;
    fn promotion_recog(self, code: &Code) -> Result<Self::Output, &'static str>;
    fn promotion(v: Box<dyn DynRecog>, code: &mut Code) -> Result<DynType, &'static str>;
    fn structure_assembling(self, code: &mut Code) -> Result<Self::Output, &'static str>;
}

impl<const N: usize> DynSeqRecog for [Box<dyn DynRecog>; N] {
    type Output = [DynType; N];
    fn promotion_recog(self, code: &Code) -> Result<Self::Output, &'static str> {
        self.structure_assembling(&mut code.clone())
    }

    fn structure_assembling(self, code: &mut Code) -> Result<Self::Output, &'static str> {
        self.try_map(|v| Self::promotion(v, code))
    }

    fn promotion(v: Box<dyn DynRecog>, code: &mut Code) -> Result<DynType, &'static str> {
        v.recog(code).map(|v| {
            code.cursor = v.span().end;
            v
        })
    }
}

pub trait DynRecog {
    fn recog(&self, code: &mut Code) -> Result<DynType, &'static str>;
}

impl<T: MarkerConversion + 'static> DynRecog for T {
    fn recog(&self, code: &mut Code) -> Result<DynType, &'static str> {
        T::conv(code).map(|v| Box::new(v) as DynType)
    }
}

/// посрденик к типовому распознаванию, так как `[Box<dyn _>; _]` требует `self`
pub trait MarkerConversion {
    type Output: CommonRecog + Spanable;
    fn conv(code: &mut Code) -> Result<Self::Output, &'static str> {
        Self::Output::recog(code)
    }
}

/// для токенов
impl<T> MarkerConversion for T
where
    Token<T>: CommonRecog,
{
    type Output = Token<T>;
}

// для последовательностей
macro_rules! dyn_impl_seq {
    (@impl $($a:ident)+) => {
        impl<$($a: MarkerConversion),+> MarkerConversion for ($($a),+) {
            type Output = ($($a::Output),+);
        }
    };
}

// tuple_impl!(dyn_impl_seq! @impl);

use items::*;
#[test]
fn test() {
    macro_rules! check {
        (($code:literal, [$($a:expr),+], [$($c:ty),+]), $b:expr) => {
            let v = [$($a as Box<dyn DynRecog>),+].promotion_recog(&$code.into()).unwrap();

            $(
                assert_eq!(
                    *v[${index()}].as_any().downcast_ref::<$c>().unwrap(),
                    $b[${index()}]
                );
            )+
        };
    }

    check!(
        (r#"tmp"n""#, [Box::new(Ident), Box::new(String)], [Token<Ident>, Token<String>]),
        [Token::new(0..3), Token::new(3..6)]
    );
    check!(
        (r#"tmp"n"tmp"#, [Box::new(IdentStringMarker)], [IdentString]),
        [IdentString(
            Token::new(0..3),
            Token::new(3..6),
            Token::new(6..9)
        )]
    );
    // check!(
    //     (r#"tmp"n""#, [Box::new((Ident, String))], [(Token<Ident>, Token<String>)]),
    //     [(Token::new(0..3), Token::new(3..6))]
    // );
}