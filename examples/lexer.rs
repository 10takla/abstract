use r#abstract::lexer::{items::Items, Code, Parse};

fn main() {
    let code = r#"sdfsf = 2 sdfsdf sdf 
    sd = 2434 sfdfsdf sdfs ffdsf sdf sdf s"#;
    println!("{}", Items::parse(&mut Code::new(code)).unwrap());

    println!(
        "{}",
        Items::parse(&mut Code::new(
            r#"abc = 2 def = "test" xyz = 100 name = "John" age = 30"#
        ))
        .unwrap()
    );

    println!(
        "{}",
        Items::parse(&mut Code::new(
            r#"
            afsg__223
            afsg___

            main {
                a = 2
                a += 200
                a -= 200
                a *= 200
                {
                    {
                        {
                            t
                        }
                    }
                }
            }

            main..

            result = 502
            "#
        ))
        .unwrap()
    );
}
