use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar);
pub mod ast;
pub mod execution;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_parser() {
        let list = grammar::ExpressionParser::new()
            .parse("tf( [56.6, 4,    -3.3], [])")
            .unwrap();
        use ast::Expression::*;
        assert_eq!(
            list,
            FunctionCall {
                function: Identifier("tf".into()).into(),
                arguments: vec![
                    VectorLiteral(vec![
                        FloatLiteral(56.6),
                        FloatLiteral(4.0),
                        FloatLiteral(-3.3)
                    ]),
                    VectorLiteral(vec![]),
                ]
            }
        );
    }
}
