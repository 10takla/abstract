pub mod call;
pub mod init;

use crate::SelectionParse;
use call::{CallBlockDistruct, CallBlockDistructDiag};
use init::{InitBlockDistruct, InitBlockDistructDiag};
use macros::Parse;

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
#[diag(BlockDistructDiag)]
pub enum BlockDistruct<'s> {
    Init(InitBlockDistruct<'s>),
    Call(CallBlockDistruct<'s>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum BlockDistructDiag {
    Init(InitBlockDistructDiag),
    Call(CallBlockDistructDiag),
}
