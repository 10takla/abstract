use super::Parse;
use crate::{Code, Recognized};
use std::fmt::Debug;
use std_reset::prelude::Deref;

#[derive(PartialEq, Debug, Clone, Deref)]
pub struct Diags<T> {
    #[deref]
    pub errors: Vec<T>,
    pub pos: Option<usize>,
    pub warrnings: Vec<()>,
}

impl<T> Extend<Diag<T>> for Diags<T> {
    fn extend<R: IntoIterator<Item = Diag<T>>>(&mut self, errors: R) {
        let (pos, errors) = Self::filter(errors.into_iter());
        if let Some(pos) = pos {
            if let Some(p) = &mut self.pos {
                if *p < pos {
                    *p = pos;
                    self.errors = errors;
                } else if *p == pos {
                    self.errors.extend(errors);
                }
            } else {
                self.pos = Some(pos);
                self.errors = errors;
            }
        }
    }
}
impl<T> Extend<T> for Diags<T> {
    fn extend<R: IntoIterator<Item = T>>(&mut self, errors: R) {
        self.errors.extend(errors);
    }
}

impl<T> Diags<T> {
    // pub fn push(&mut self, (pos, error): Diag<T>) {
    //     if let Some(p) = &mut self.pos {
    //         if *p < pos {
    //             *p = pos;
    //             self.errors = vec![error];
    //         } else if *p == pos {
    //             self.errors.push(error);
    //         }
    //     } else {
    //         self.pos = Some(pos);
    //         self.errors = vec![error];
    //     }
    // }

    // pub fn extend(&mut self, ext: impl Iterator<Item = Diag<T>> + Debug) {
    //     let (pos, errors) = Self::filter(ext);
    //     if let Some(pos) = pos {
    //         if let Some(p) = &mut self.pos {
    //             if *p < pos {
    //                 *p = pos;
    //                 self.errors = errors;
    //             } else if *p == pos {
    //                 self.errors.extend(errors);
    //             }
    //         } else {
    //             self.pos = Some(pos);
    //             self.errors = errors;
    //         }
    //     }
    // }

    pub fn from_errors(errors: Vec<Diag<T>>) -> Self {
        if errors.is_empty() {
            return Self::default();
        }
        let (pos, errors) = Self::filter(errors.into_iter());

        Self {
            errors,
            pos,
            ..Default::default()
        }
    }
    fn filter(errors: impl Iterator<Item = Diag<T>>) -> (Option<usize>, Vec<T>) {
        let mut stack = vec![];
        let mut max = None;
        for (i, error) in errors {
            match &mut max {
                None => {
                    stack = vec![error];
                    max = Some(i);
                }
                Some(max) if i > *max => {
                    stack = vec![error];
                    *max = i;
                }
                Some(max) if i == *max => {
                    stack.push(error);
                }
                _ => {}
            }
        }
        (max, stack)
    }
}

impl<T> Default for Diags<T> {
    fn default() -> Self {
        Self {
            errors: Default::default(),
            warrnings: Default::default(),
            pos: Default::default(),
        }
    }
}

pub type Diag<T> = (usize, T);

pub trait DiagParse<'s>: Parse<'s> {
    fn diag(code: &Code<'s>, recognized: &mut Recognized<'s>) -> Result<Self, Self::Diags> {
        let mut diags = Default::default();
        Self::parse(code, &mut diags, recognized).ok_or(diags)
    }

    fn diag_and_consume(
        code: &mut Code<'s>,
        recognized: &mut Recognized<'s>,
    ) -> Result<Self, Self::Diags> {
        Self::diag(code, recognized).map(|v| v.consume(code, recognized))
    }
}
