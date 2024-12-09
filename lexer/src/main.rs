use colored::Colorize;
use lexer::parse;

fn main() {
    let source = r#"abc 43 c2 78 dd 22s  22s"#;
    let errors = parse(source).1;
    for (i, error) in errors {
        let source = source.chars().collect::<Vec<_>>();
        let (l, b, [min, max]) = (" |".blue(), "...".blue(), [10, 4]);

        let code = format!(
            "{}{}{}",
            source[if i < min { 0 } else { i - min }..i]
                .iter()
                .collect::<String>(),
            source[i].to_string().underline().red(),
            source[i + 1..if source.len() - 1 - i < max {
                source.len()
            } else {
                i + max
            }]
                .iter()
                .collect::<String>()
        );

        println!(
            "
{l}
{l} {b}{code}
{l} {}{}
",
            " ".repeat(min + b.chars().count()),
            format!("{}-Ожидается {error:?}", "^".repeat(1)).red()
        );
    }
}
