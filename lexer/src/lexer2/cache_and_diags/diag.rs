use crate::{
    lexer2::{
        code::Source, tests::cli_args, Construct, ConstructParse, ErrorType, Items, ParseArgs,
        Slice,
    },
    parse,
};
use colored::Colorize;
use macros::{constructor, parse_test};
use paste::paste;
use regex::Regex;
use regex_automata::{
    dfa::{dense::DFA, Automaton},
    nfa::thompson::NFA,
    util::start::Config,
    Input,
};
use regex_syntax::{ast::parse::Parser as AstParser, Parser};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::DefaultHasher,
    io::{self, stdout, Cursor, Read, Write},
    ops::RangeInclusive,
    option::Iter,
    rc::Rc,
    sync::{Arc, Mutex},
    vec::IntoIter,
};
use std_reset::prelude::Deref;
use tracing::info;

#[derive(Clone, Debug, Deref)]
pub struct Diag {
    #[deref]
    pub slice: Slice,
    pub source: Source,
    pub error: ErrorType,
}

impl std::fmt::Display for Diag {
    fn fmt(&self, f_: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Diag {
            source: Source {
                real_source: source,
                ..
            },
            slice,
            error,
        } = self;

        let get_line = |pos| {
            let mut acc = 0;
            source
                .split_inclusive('\n')
                .enumerate()
                .find_map(|(i, str)| {
                    acc += str.chars().count();
                    (pos < acc).then_some(i + 1)
                })
                .unwrap_or_default()
        };

        let f = |v: &[char]| v.iter().collect::<std::string::String>();

        let source = source.chars().collect::<Vec<_>>();
        let (l, b, [min, max]) = ("|".blue(), "...", [0, 4]);
        let [distr_after, distr_before] = [
            *slice.start() > min,
            source.len() - slice.end() != 0 && source.len() - 1 - slice.end() >= max,
        ]
        .map(|cond| cond.then_some(b).unwrap_or_default());

        let code = format!(
            "{}{}{}",
            f(&source[{
                let i = *slice.start();
                let r = if i < min { 0 } else { i - min };
                r..i
            }]),
            f(&source
                .get(slice.clone())
                .unwrap_or(&[*source.last().unwrap()]))
            .underline()
            .red(),
            f(&source[{
                let i = *slice.end();
                if source.len() - i == 0 {
                    i..source.len()
                } else {
                    i + 1..if source.len() - 1 - i < max {
                        source.len()
                    } else {
                        i + max
                    }
                }
            }])
        );

        let front_p = 3;
        let f = " ".repeat(front_p);

        writeln!(
            f_,
            "
{f}{l}
{}{l} {}{code}{}
{f}{l} {}{}
",
            format!("{:width$} ", get_line(*slice.end()), width = front_p - 1),
            distr_after.blue(),
            distr_before.blue(),
            " ".repeat(min + distr_after.chars().count()),
            format!(
                "{}-Ожидается {error:?}",
                "^".repeat(slice.end() - slice.start() + 1),
            )
            .red()
        )
    }
}

#[parse_test]
fn display(print: crate::lexer2::print::Print) {
    let mut code = (
        r#"2fdg  2fdg      
    2fdg"#,
        print,
    )
        .into();
    Items::recog(&mut code, 0);
    for diag in code.c_a_d.clone().borrow().errors.clone() {
        println!("{diag}");
    }
}

mod issues {
    use crate::lexer2::{print::Print, tests::cli_args, Items};
    use macros::parse_test;

    // Ошибка происходит из-за необработки в `Diag::display` диапазона ошибки, выходящей за границу кода
    // Подробно: Когда код заканчивается, но при этом требуется завершение обработки элементов конструкции.
    // Например, как в данном случае обработка Block: удачно обрабатывается `{`, далее код заканчивается, поэтому ошибка заверщающей коснтрукцию элемента `}` должна находится после `{`, то есть за гранцией строки кода.
    // Примечание: Поэтому чисто семантически было бы правильно указывать диапазон ошибки за кодом
    // Исправление: Из примечания следует, что нужно обрабатывать случаи выхода ошибки за границу кода на уровне `Diag::display`
    #[parse_test]
    fn outside_boundary_code(print: Print) {
        let mut code = (r#"{"#, print).into();
        Items::recog(&mut code, 0);
        for diag in code.c_a_d.clone().borrow().errors.clone() {
            // dbg!(&diag);
            println!("{diag}");
        }
    }

    #[parse_test]
    fn invalid_error_range(print: Print) {
        let mut code = (
            r#"
22dd"#, print,
        )
            .into();
        Items::recog(&mut code, 0);
        for diag in code.c_a_d.clone().borrow().errors.clone() {
            // dbg!(&diag);
            println!("{diag}");
        }
    }
}
