use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar);
pub mod ast;
pub mod execution;

#[cfg(test)]
mod tests {
    use ast::Expression;

    use super::*;

    #[test]
    fn expression_parser() {
        let list = grammar::ExpressionParser::new()
            .parse("tf( [56.6 4    -3.3], [])")
            .unwrap();
        assert_eq!(
            list,
            Expression::TransferFunction(vec![56.6, 4.0, -3.3], vec![])
        );
    }
}
