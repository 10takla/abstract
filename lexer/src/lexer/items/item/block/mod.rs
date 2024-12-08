pub mod distruct;
pub mod init;

use crate::lexer::{check_diag, DiagParse, Slicable};
use distruct::{BlockDistruct, BlockDistructDiag};
use init::{InitBlock, InitBlockDiag};
use macros::Parse;

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
#[diag(BlockDiag)]
pub enum Block<'s> {
    Distruct(BlockDistruct<'s>),
    Init(InitBlock<'s>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum BlockDiag {
    Distruct(BlockDistructDiag),
    Init(InitBlockDiag),
}