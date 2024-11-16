use r#abstract::compile;
use std::{fs::File, io::Read};

fn main() {
    let mut source = String::new();
    File::open("./examples/code.abs")
        .unwrap()
        .read_to_string(&mut source)
        .unwrap();
    println!("{}", compile(&source).unwrap());
}
