use super::*;
use clap::Parser;
use macros::parse_test;
use std::env::{self, args};
use tracing::{dispatcher::with_default, level_filters::LevelFilter, Level};
use tracing_subscriber::fmt::format;

#[parse_test]
fn tmp(print: Print) {
    // let mut t = "fn ываыва( dsfs, dsfs, ){sdfsdf}".into();
    // dbg!(FnHead::recog(&mut t));

    let mut t = ("crate::sdfsdf::sdfsdf::", print).into();
    dbg!(Path::recog(&mut t, 0));
}

mod cache {
    use super::*;
    mod pass {
        use super::*;
        mod construct {
            use super::*;
            use macros::parse_test;
            // кеширование головы конструкции. DistructBlock должен парсится 1 раз
            #[parse_test]
            fn head(print: Print) {
                let mut t = ("main..", print).into();
                dbg!(CacheConstructHead::recog(&mut t, 0));
            }

            // кеширование элементов конструкции. элемнты Var2 должны распознаться только 1 раз до Var2
            #[parse_test]
            fn item(print: Print) {
                let mut t = (
                    "maijn\t=2323 main { }..sdf\nr+=2 t/=4\"sdfsf sdf\"-+=+",
                    print,
                )
                    .into();
                // let mut t = "mai { }..".into();
                dbg!(CacheConstructItem::recog(&mut t, 0));
            }

            // После Var7 `goods` от Var6 должны расширятся, а не создаватбся новый список
            #[parse_test]
            fn list_walkthrough(print: Print) {
                let mut t = ("main}{ ", print).into();
                dbg!(CacheConstructWalkthroug::recog(&mut t, 0));
            }

            mod with {
                use super::*;
                use macros::parse_test;

                // Ident должен распознатся 1 раз, как токен без конструкции но часть вариации
                #[parse_test]
                fn token(print: Print) {
                    let mut t = (r#"sdfsdf1."#, print).into();
                    // 0 -> NamedBlock -> Ident Block -> Fail -> memeory (pos, item)
                    // 0 -> Ident -> (x from memeory)
                    dbg!(CacheToken::recog(&mut t, 0));
                }

                #[parse_test]
                fn enum_(print: Print) {
                    let mut t = ("+ ", print).into();
                    dbg!(CacheEnum::recog(&mut t, 0));
                }
            }
        }
    }
    mod fail {
        use super::*;
        use macros::parse_test;

        #[parse_test]
        fn token(print: Print) {
            let mut t = ("2sdfsfd", print).into();
            dbg!(CacheToken::recog(&mut t, 0));
        }

        #[parse_test]
        fn enum_(print: Print) {
            let mut t = ("22", print).into();
            dbg!(CacheEnum::recog(&mut t, 0));
        }
    }
}

mod errors {
    use super::*;
    use macros::parse_test;

    #[parse_test]
    fn any(print: Print) {
        let mut t = (r#"  "sdfsf "#, print).into();
        dbg!([Construct::WhiteSpace, Construct::String].recog(&mut t, 0));
    }

    #[parse_test]
    fn enum_errors(print: Print) {
        let e = dbg!(Op::recog(&mut (r#"2"#, print).into(), 0)).unwrap_err();
        let ErrorType::Op(diags) = e.error else {
            panic!()
        };
        
        assert_eq!(
            diags
                .into_iter()
                .map(|v| (v.slice, v.error, v.type_))
                .collect::<Vec<_>>(),
            vec![
                (0..=0, ErrorType::Reg("\\+",), Construct::Add),
                (0..=0, ErrorType::Reg("-"), Construct::Sub),
                (0..=0, ErrorType::Reg("\\*"), Construct::Mul),
                (0..=0, ErrorType::Reg("/"), Construct::Div)
            ]
        );
    }

    mod error_passing {
        use super::*;
        use clap::Id;

        #[parse_test]
        fn assign_expr(print: Print) {
            let mut args = (r#"sdfsfd -= "#, print).into();
            dbg!(Items::recog(&mut args, 0));
            let e = args.c_a_d.borrow().errors[0].clone();
            assert_eq!(
                (*e.slice.end(), e.error, e.type_),
                (10, ErrorType::LineOver, Construct::Literal)
            );
        }

        #[parse_test]
        fn plus(print: Print) {
            let mut args: ParseArgs = (r#"+"#, print).into();
            dbg!(Items::recog(&mut args, 0));
            dbg!(&args.c_a_d.borrow().errors);
        }
    }
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

mod items {
    use super::*;

    #[parse_test]
    fn path(print: Print) {
        macro_rules! check {
            ($l:literal, $($type_:tt)*) => {
                let mut t = ($l, print.clone()).into();
                assert!(
                    match Path::recog(&mut t, 0) {
                        $($type_)* => true,
                        _ => false,
                    }
                );
            };
        }

        check!(
            "crate::sdfsdf::sdfsdf",
            Ok(Path::CurrenPath(CurrenPath(CurrentPathV::Crate(_), ..)))
        );
        check!(
            "self::sdfsdf::sdfsdf",
            Ok(Path::CurrenPath(CurrenPath(CurrentPathV::Self_(_), ..)))
        );
        check!(
            "super::sdfsdf::sdfsdf",
            Ok(Path::CurrenPath(CurrenPath(CurrentPathV::Super(_), ..)))
        );
        check!(
            "sdfsdf::sdfsdf::sdfsfdf",
            Ok(Path::CurrenPath(CurrenPath(CurrentPathV::Ident(_), ..)))
        );
        check!("::sdfsdf::sdfsdf", Ok(Path::EndPath(_)));

        check!(
            "sdfsdf::sdfsdf::*",
            Ok(Path::CurrenPath(CurrenPath(
                _,
                EndPath::WithItemsEnd(WithItemsEnd(_, _, PathItemEnd::GlobImport(_)))
            )))
        );

        check!(
            "sdfsdf::sdfsdf::{ *, }",
            Ok(Path::CurrenPath(
                CurrenPath(_, EndPath::WithItemsEnd(WithItemsEnd(
                        _,
                        _,
                        PathItemEnd::PathItemsC(PathItemsC(_, PathItemsI(vec), _))
                    )
                ))
            )) if let PathItemC(PathItemV::GlobImport(_), ..) = vec[0]
        );
        check!(
            "sdfsdf::sdfsdf::{ self, }",
            Ok(Path::CurrenPath(
                CurrenPath(_, EndPath::WithItemsEnd(WithItemsEnd(
                        _,
                        _,
                        PathItemEnd::PathItemsC(PathItemsC(_, PathItemsI(vec), _))
                    )
                ))
            )) if let PathItemC(PathItemV::Self_(_), ..) = vec[0]
        );
        check!(
            "sdfsdf::sdfsdf::{ sdfsdf, }",
            Ok(Path::CurrenPath(
                CurrenPath(_, EndPath::WithItemsEnd(WithItemsEnd(
                        _,
                        _,
                        PathItemEnd::PathItemsC(PathItemsC(_, PathItemsI(vec), _))
                    )
                ))
            )) if let PathItemC(PathItemV::Ident(_), ..) = vec[0]
        );
    }
}

mod issues {
    use super::{Code, Literal, ParseArgs, Print, Source};
    use crate::lexer2::{tests::cli_args, Items};
    use clap::Parser;
    use macros::parse_test;

    // Когда происходит ошибка String::EndsWithQuote она происходит до конца кода, при этом items продолжают дальше распозноваться
    // Ошибка происходила из-за то что мы инкрементируем к cursor длинну ошибки, которая всегда получалась минимум 1, так как было:
    // ```arg.code.cursor += e.end() - i + 1;```
    // Исправление: Замена на ```arg.code.cursor = e.end();```
    // Примечание: Также исправление является лучшим вариантом для смещения cursor, так как требует меньше опреаций
    #[parse_test]
    fn items_with_string_error(print: Print) {
        Items::recog(&mut (r#"{"22sd}"#, print).into(), 0);
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
        #[arg(long, default_value_t = false)]
        pub(super) cache: bool,
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
        cache: args.cache,
        ..Default::default()
    }
}
