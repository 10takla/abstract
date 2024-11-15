pub mod distruct;
pub mod init;

use crate::lexer::{Parse, Slicable};
use distruct::Distruct;
use init::Init;
use macros::Parse;

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
pub enum Block {
    Distruct(Distruct),
    Init(Init),
}
