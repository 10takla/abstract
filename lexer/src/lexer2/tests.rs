use super::*;
use std::env;
use tracing::{dispatcher::with_default, level_filters::LevelFilter, Level};
use tracing_subscriber::fmt::format;

#[test]
fn tmp() {
    setup_tracing();
    // let mut t = "fn ываыва( dsfs, dsfs, ){sdfsdf}".into();
    // dbg!(FnHead::recog(&mut t));

    let mut t = "fn ываыва
    
    { dsfs: Ar, dsfs: Ty, }{sdfsdf}".into();
    dbg!(FnHead::recog(&mut t));
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
                setup_tracing();
                let mut t = "main..".into();
                dbg!(CacheConstructHead::recog(&mut t));
            }

            /// кеширование элементов конструкции. элемнты Var2 должны распознаться только 1 раз до Var2
            #[test]
            fn item() {
                setup_tracing();
                let mut t = "maijn\t=2323 main { }..sdf\nr+=2 t/=4\"sdfsf sdf\"-+=+".into();
                // let mut t = "mai { }..".into();
                dbg!(CacheConstructItem::recog(&mut t));
            }

            /// После Var7 `goods` от Var6 должны расширятся, а не создаватбся новый список
            #[test]
            fn list_walkthrough() {
                setup_tracing();
                let mut t = "main}{ ".into();
                dbg!(CacheConstructWalkthroug::recog(&mut t));
            }

            mod with {
                use super::*;
                /// Ident должен распознатся 1 раз, как токен без конструкции но часть вариации
                #[test]
                fn token() {
                    setup_tracing();
                    let mut t = r#"sdfsdf1."#.into();
                    // 0 -> NamedBlock -> Ident Block -> Fail -> memeory (pos, item)
                    // 0 -> Ident -> (x from memeory)
                    dbg!(CacheToken::recog(&mut t));
                }

                #[test]
                fn enum_() {
                    setup_tracing();
                    let mut t = "+ ".into();
                    dbg!(CacheEnum::recog(&mut t));
                }
            }
        }
    }
    mod fail {
        use super::*;

        #[test]
        fn token() {
            setup_tracing();
            let mut t = "2sdfsfd".into();
            dbg!(CacheToken::recog(&mut t));
        }

        #[test]
        fn enum_() {
            setup_tracing();
            let mut t = "22".into();
            dbg!(CacheEnum::recog(&mut t));
        }
    }
}

mod errors {
    use super::*;

    #[test]
    fn items() {
        setup_tracing();
        let mut t = r#"  "sdfsf "#.into();
        dbg!([Construct::WhiteSpace, Construct::String].recog(&mut t));
    }
}

#[test]
fn code() {
    setup_tracing();
    dbg!(Items::recog(
        &mut r#"
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
        .into()
    ));
}

pub fn setup_tracing() {
    if env::var("LOG_LEVEL") == Ok("INFO".into()) {
        tracing_subscriber::fmt()
            .with_max_level(LevelFilter::INFO)
            .with_writer(std::io::stdout) // Указываем вывод в стандартный вывод
            .fmt_fields(format::PrettyFields::new()) // Простой и понятный формат полей
            .event_format(format::Format::default().compact())
            .with_target(false)
            .without_time() // Компактный формат без времени и уровня логирования
            .try_init()
            .ok();
    }
}
