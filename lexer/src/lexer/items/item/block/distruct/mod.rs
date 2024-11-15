pub mod init;
pub mod named;

use init::InitBlockDistruct;
use macros::Parse;
use named::CallBlockDistruct;

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
pub enum Distruct<'s> {
    Init(InitBlockDistruct<'s>),
    Call(CallBlockDistruct<'s>),
}
