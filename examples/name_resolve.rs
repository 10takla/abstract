use r#abstract::{
    compile,
    lexer::{items::Items, Code, Parse},
};

fn main() {
    compile(
        r#"
           
            main {
                a = 2
                a += 200
                a -= 200
                a *= 200
                t = 10
                {
                    {
                        {
                            t
                        }
                    }
                }
            }..

            main..

            result = 502
            "#,
    );
}
