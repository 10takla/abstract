use super::*;

mod cache {
    use super::*;
    mod pass {
        use super::*;
        mod construct {
            use super::*;
            /// кеширование головы конструкции. DistructBlock должен парсится 1 раз
            #[test]
            fn head() {
                let mut t = "main..".into();
                dbg!(CacheConstructHead::recog(&mut t, 0));
            }

            /// кеширование элементов конструкции. элемнты Var2 должны распознаться только 1 раз до Var2
            #[test]
            fn item() {
                let mut t = "maijn\t=2323 main { }..sdf\nr+=2 t/=4\"sdfsf sdf\"-+=+".into();
                // let mut t = "mai { }..".into();
                dbg!(CacheConstructItem::recog(&mut t, 0));
            }

            /// После Var7 `goods` от Var6 должны расширятся, а не создаватбся новый список
            #[test]
            fn list_walkthrough() {
                let mut t = "main}{ ".into();
                dbg!(CacheConstructWalkthroug::recog(&mut t, 0));
            }

            mod with {
                use super::*;
                /// Ident должен распознатся 1 раз, как токен без конструкции но часть вариации
                #[test]
                fn token() {
                    let mut t = r#"sdfsdf1."#.into();
                    // 0 -> NamedBlock -> Ident Block -> Fail -> memeory (pos, item)
                    // 0 -> Ident -> (x from memeory)
                    dbg!(CacheToken::recog(&mut t, 0));
                }

                #[test]
                fn enum_() {
                    let mut t = "+ ".into();
                    dbg!(CacheEnum::recog(&mut t, 0));
                }
            }
        }
    }
    mod fail {
        use super::*;

        #[test]
        fn token() {
            let mut t = "2sdfsfd".into();
            dbg!(CacheToken::recog(&mut t, 0));
        }

        #[test]
        fn enum_() {
            let mut t = "22".into();
            dbg!(CacheEnum::recog(&mut t, 0));
        }
    }
}

mod errors {
    use super::*;

    #[test]
    fn items() {
        let mut t = r#"  "sdfsf "#.into();
        dbg!([Construct::WhiteSpace, Construct::String].recog(&mut t, 0));
    }
}

#[test]
fn code() {
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
        .into(),
        0
    ));
}