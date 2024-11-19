use ast::name_resolve::name_resolve;
use lexer::{items::Items, parse, Code, DiagParse};

fn main() {
    let items = parse(
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
    let (refs, _) = name_resolve(&items, None);
}
