pub mod distruct;
pub mod init;

use crate::lexer::{check_diag, DiagParse, Slicable};
use distruct::{Distruct, DistructDiag};
use init::{Init, InitDiag};
use macros::Parse;

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
#[diag(BlockDiag)]
pub enum Block<'s> {
    Distruct(Distruct<'s>),
    Init(Init<'s>),
}

#[derive(PartialEq, Debug)]
pub enum BlockDiag {
    Distruct(DistructDiag),
    Init(InitDiag),
}