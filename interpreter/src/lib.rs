use ast::{Expression, Program};
use engine::{state_space::DiscreteStateSpaceModel, transfer_function::DiscreteTransferFunction};
use lalrpop_util::lalrpop_mod;
use nalgebra::DVector;
use std::{collections::HashMap, fmt::Write};

lalrpop_mod!(pub grammar);
pub mod ast;

#[derive(Clone, Debug, PartialEq)]
pub enum ExecutionError {
    IO(std::fmt::Error),
    NullDeref,
    TypeError,
    Other(String),
}

impl From<std::fmt::Error> for ExecutionError {
    fn from(value: std::fmt::Error) -> Self {
        Self::IO(value)
    }
}

/// Runtime Value
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    TransferFunction(DiscreteTransferFunction),
    StateSpaceModel(DiscreteStateSpaceModel),
}

pub fn execute(program: &Program) -> Result<String, ExecutionError> {
    use ast::Statement::*;
    let mut output = String::new();
    let mut values: HashMap<String, Value> = HashMap::new();
    for stmt in &program.statements {
        match stmt {
            ExpressionStatement(expr) => {
                eval(expr, &values, &mut output)?;
            }
            Assign(id, expr) => {
                let value = eval(expr, &values, &mut output)?;
                values.insert(id.to_string(), value);
            }
        }
    }
    Ok(output)
}

fn eval(
    expr: &Expression,
    env: &HashMap<String, Value>,
    output: &mut String,
) -> Result<Value, ExecutionError> {
    use Expression::*;
    let value = match expr {
        TransferFunction(num, den) => {
            let num = DVector::from_vec(num.clone());
            let den = DVector::from_vec(den.clone());
            let tf = DiscreteTransferFunction::new(num, den)
                .ok_or(ExecutionError::Other("Could not construct tf".to_string()))?;
            writeln!(output, "{tf}")?;
            Value::TransferFunction(tf)
        }
        TF2SS(tf) => {
            let tf = env.get(tf).ok_or(ExecutionError::NullDeref)?;
            let Value::TransferFunction(tf) = tf else {
                return Err(ExecutionError::TypeError);
            };
            let ss = tf.convert_to_state_space().ok_or(ExecutionError::Other(
                "Could not convert to state space".to_string(),
            ))?;
            writeln!(output, "{ss}")?;
            Value::StateSpaceModel(ss)
        }
    };
    Ok(value)
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
