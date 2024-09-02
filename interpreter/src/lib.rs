use ast::Program;
use engine::transfer_function::DiscreteTransferFunction;
use lalrpop_util::lalrpop_mod;
use nalgebra::DVector;
use std::fmt::Write;

lalrpop_mod!(pub grammar);
pub mod ast;

#[derive(Clone, Debug, PartialEq)]
pub enum ExecutionError {
    IO(std::fmt::Error),
    Other(String),
}

pub fn execute(program: &Program) -> Result<String, ExecutionError> {
    use ast::Expression::*;
    use ast::Statement::*;
    let mut output = String::new();
    for stmt in &program.statements {
        match stmt {
            Assign(_, expr) => match expr {
                TransferFunction(num, den) => {
                    let num = DVector::from_vec(num.clone());
                    let den = DVector::from_vec(den.clone());
                    let tf = DiscreteTransferFunction::new(num, den)
                        .ok_or(ExecutionError::Other("Could not construct tf".to_string()))?;
                    writeln!(output, "{tf}").map_err(|e| ExecutionError::IO(e))?;
                }
            },
        }
    }
    Ok(output)
}

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
