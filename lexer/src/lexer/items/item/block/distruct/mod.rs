pub mod init;
pub mod named;

use init::{InitBlockDistruct, InitBlockDistructDiag};
use macros::Parse;
use named::{CallBlockDistruct, CallBlockDistructDiag};

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
#[diag(DistructDiag)]
pub enum Distruct<'s> {
    Init(InitBlockDistruct<'s>),
    Call(CallBlockDistruct<'s>),
}

#[derive(PartialEq, Debug)]
pub enum DistructDiag {
    Init(InitBlockDistructDiag),
    Call(CallBlockDistructDiag),
}
