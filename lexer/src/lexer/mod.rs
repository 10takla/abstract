pub mod items;
// mod recognize;
mod shared;

pub use shared::{
    code::*,
    parse::{diag::*, recognizee::*, *},
    slice::*,
    test_funcs::*,
    *
};
