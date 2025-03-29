use crate::peg_grammar;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::{
    env,
    fs::read_to_string,
    path::{Path, PathBuf},
};
use syn::{parse_macro_input, LitStr};

pub fn parse_from_peg_file(input: TokenStream) -> TokenStream {
    let full_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(&parse_macro_input!(input as LitStr).value());
    let file_content = read_to_string(&full_path)
        .unwrap_or_else(|_| panic!("Не удалось прочитать файл по пути {}.", full_path.display()));
    peg_grammar(file_content.parse().unwrap())
}
