vec![
    Ident(asfsg),
    Ident(asfsg),
    Block::Named::Identifier::Base(
        name: Ident(main),
        block: vec![
            Assign ( LeftRight { left: Ident(a), right: Literal::Number(2) } ),
            AssignAnd::Add( LeftRight { left: Ident(a), right: Literal::Number(200) } ),
            AssignAnd::Sub( LeftRight { left: Ident(a), right: Literal::Number(200) } ),
            AssignAnd::Mul( LeftRight { left: Ident(a), right: Literal::Number(200) } ),
            Assign ( LeftRight { left: Ident(t), right: Block::Named::Identifier::Disturctered {
                name: Ident(m),
                block: vec![
                    AssignAnd::Add( LeftRight { left: Ident(a), right: Literal::Number(300) } ),
                    Ident(a),
                ]
            } } ),
            Block::Unnamed(vec![
                Block::Unnamed(vec![
                    Block::Unnamed(vec![
                        Ident(t)
                    ])
                ])
            ])
        ]
    ),
    Block::Named::Disturctered(Ident(main)),
    Assign ( LeftRight { left: Ident(result), right: Literal::Number(502) } )
]