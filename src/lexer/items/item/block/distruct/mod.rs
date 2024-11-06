pub mod init;
pub mod named;

use init::InitBlockDistruct;
use macros::Parse;
use named::BlockDistruct;

#[derive(PartialEq, Debug, Parse)]
pub enum DistructBlock {
    Named(BlockDistruct),
    Init(InitBlockDistruct),
}
