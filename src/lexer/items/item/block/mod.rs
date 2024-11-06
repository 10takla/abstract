pub mod distruct;
pub mod init;

use crate::lexer::{Parse, Slicable};
use distruct::DistructBlock;
use init::InitBlock;
use macros::Parse;

#[derive(PartialEq, Debug, Parse)]
pub enum Block {
    InitBlock(InitBlock),
    DistructBlock(DistructBlock),
}
