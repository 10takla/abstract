use crate::{
    lexer2::{code::Source, tests::setup_tracing, Construct, ConstructParse, ErrorType, Items, ParseArgs, Slice},
    parse,
};
use colored::Colorize;
use macros::constructor;
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
                .unwrap()
        };

        let f = |v: &[char]| v.iter().collect::<std::string::String>();

        let source = source.chars().collect::<Vec<_>>();
        let (l, b, [min, max]) = ("|".blue(), "...", [0, 4]);
        let [distr_after, distr_before] =
            [*slice.start() == 0, source.len() - 1 - slice.end() < max].map(|cond| {
                if cond {Default::default()} else {b}
            });

        let code = format!(
            "{}{}{}",
            f(&source[{
                let i = *slice.start();
                let r = if i < min { 0 } else { i - min };
                r..i
            }]),
            f(&source[slice.clone()]).underline().red(),
            f(&source[{
                let i = slice.end();
                i + 1..if source.len() - 1 - i < max {
                    source.len()
                } else {
                    i + max
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
            dbg!(distr_before).blue(),
            " ".repeat(min + distr_after.chars().count()),
            format!(
                "{}-Ожидается {error:?}",
                "^".repeat(slice.end() - slice.start() + 1),
            )
            .red()
        )
    }
}

#[test]
fn display() {
    setup_tracing();
    let mut t: ParseArgs = r#"2fdg  2fdg      
    2fdg"#.into();
    Items::recog(&mut t, 0);
    for diag in t.c_a_d.clone().borrow().errors.clone() {
        println!("{diag}");
    }
}
