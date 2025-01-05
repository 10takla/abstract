use super::*;
use clap::Parser;
use std::env::{self, args};
use tracing::{dispatcher::with_default, level_filters::LevelFilter, Level};
use tracing_subscriber::fmt::format;

#[test]
fn tmp() {
    // let mut t = "fn ываыва( dsfs, dsfs, ){sdfsdf}".into();
    // dbg!(FnHead::recog(&mut t));

    let mut t = (
        "fn ываыва
    
    { dsfs: Ar, dsfs: Ty, }{sdfsdf}",
        cli_args(),
    )
        .into();
    dbg!(FnHead::recog(&mut t, 0));
}

mod cache {
    use super::*;
    mod pass {
        use super::*;
        mod construct {
            use super::*;
            /// кеширование головы конструкции. DistructBlock должен парсится 1 раз
            #[test]
            fn head() {
                let mut t = ("main..", cli_args()).into();
                dbg!(CacheConstructHead::recog(&mut t, 0));
            }

            /// кеширование элементов конструкции. элемнты Var2 должны распознаться только 1 раз до Var2
            #[test]
            fn item() {
                let mut t = (
                    "maijn\t=2323 main { }..sdf\nr+=2 t/=4\"sdfsf sdf\"-+=+",
                    cli_args(),
                )
                    .into();
                // let mut t = "mai { }..".into();
                dbg!(CacheConstructItem::recog(&mut t, 0));
            }

            /// После Var7 `goods` от Var6 должны расширятся, а не создаватбся новый список
            #[test]
            fn list_walkthrough() {
                let mut t = ("main}{ ", cli_args()).into();
                dbg!(CacheConstructWalkthroug::recog(&mut t, 0));
            }

            mod with {
                use super::*;
                /// Ident должен распознатся 1 раз, как токен без конструкции но часть вариации
                #[test]
                fn token() {
                    let mut t = (r#"sdfsdf1."#, cli_args()).into();
                    // 0 -> NamedBlock -> Ident Block -> Fail -> memeory (pos, item)
                    // 0 -> Ident -> (x from memeory)
                    dbg!(CacheToken::recog(&mut t, 0));
                }

                #[test]
                fn enum_() {
                    let mut t = ("+ ", cli_args()).into();
                    dbg!(CacheEnum::recog(&mut t, 0));
                }
            }
        }
    }
    mod fail {
        use super::*;

        #[test]
        fn token() {
            let mut t = ("2sdfsfd", cli_args()).into();
            dbg!(CacheToken::recog(&mut t, 0));
        }

        #[test]
        fn enum_() {
            let mut t = ("22", cli_args()).into();
            dbg!(CacheEnum::recog(&mut t, 0));
        }
    }
}

mod errors {
    use super::*;

    #[test]
    fn any() {
        let mut t = (r#"  "sdfsf "#, cli_args()).into();
        dbg!([Construct::WhiteSpace, Construct::String].recog(&mut t, 0));
    }

    // #[test]
    // fn literal() {
    //     cli_args();
    //     dbg!(parse(r#"sdfsfd +"#));
    // }
}

#[test]
fn code() {
    dbg!(Items::recog(
        &mut (
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
            "#,
            cli_args()
        )
            .into(),
        0
    ));
}

mod issues {
    use super::{Code, Literal, ParseArgs, Print, Source};
    use crate::lexer2::{tests::cli_args, Items};
    use clap::Parser;

    /// Когда происходит ошибка String::EndsWithQuote она происходит до конца кода, при этом items продолжают дальше распозноваться
    /// Ошибка происходила из-за то что мы инкрементируем к cursor длинну ошибки, которая всегда получалась минимум 1, так как было:
    /// ```arg.code.cursor += e.end() - i + 1;```
    /// Исправление: Замена на ```arg.code.cursor = e.end();```
    /// Примечание: Также исправление является лучшим вариантом для смещения cursor, так как требует меньше опреаций
    #[test]
    fn items_with_string_error() {
        Items::recog(&mut (r#"{"22sd}"#, cli_args()).into(), 0);
    }
}

pub fn cli_args() -> Print {
    use std::string::String;

    #[derive(Clone, Debug, Default, Parser)]
    #[command(about, rename_all = "kebab-case")]
    pub struct Args {
        #[arg(long, action, default_value_t = false)]
        pub(super) logs: bool,
        #[arg(long, default_value_t = 0)]
        pub(super) fail_level: usize,
    }

    let args = Args::parse_from({
        let mut find = false;
        args()
            .into_iter()
            .enumerate()
            .skip_while(move |(i, v)| {
                if v == "--" {
                    find = true
                }
                !find
            })
            .map(|(_, v)| v)
            .collect::<Vec<_>>()
    });

    if args.logs {
        tracing_subscriber::fmt()
            .with_max_level(LevelFilter::INFO)
            .with_writer(std::io::stdout) // Указываем вывод в стандартный вывод
            .fmt_fields(format::PrettyFields::new()) // Простой и понятный формат полей
            .event_format(format::Format::default().compact())
            .with_target(false)
            .with_level(false)
            .without_time() // Компактный формат без времени и уровня логирования
            .try_init()
            .ok();
    }

    Print {
        max_fail_level: args.fail_level,
        ..Default::default()
    }
}
