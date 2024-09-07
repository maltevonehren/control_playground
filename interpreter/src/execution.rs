use std::collections::HashMap;

use engine::state_space::DiscreteStateSpaceModel;
use engine::transfer_function::DiscreteTransferFunction;
use nalgebra::{DVector, Matrix2xX};

use crate::ast::{Expression, Program, Statement};

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    IO(std::fmt::Error),
    NullDeref,
    TypeError,
    Other(String),
}

impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Self::IO(value)
    }
}

/// Runtime Value
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    String(String),
    TransferFunction(DiscreteTransferFunction),
    StateSpaceModel(DiscreteStateSpaceModel),
    Matrix(Matrix2xX<f64>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    Err(Error),
    Text(String),
    Plot(Matrix2xX<f64>),
}

impl From<&Value> for Output {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(s) => Output::Text(s.clone()),
            Value::TransferFunction(tf) => Output::Text(format!("{tf}")),
            Value::StateSpaceModel(ss) => Output::Text(format!("{ss}")),
            Value::Matrix(data) => Output::Plot(data.clone()),
        }
    }
}

pub trait Env {
    fn read_file(&self, name: &str) -> Option<String>;
}

pub fn execute(program: &Program, exec_env: &impl Env) -> Vec<Output> {
    use Statement::*;
    let mut output = Vec::new();
    let mut values: HashMap<String, Value> = HashMap::new();
    for stmt in &program.statements {
        match stmt {
            ExpressionStatement(expr) => match eval(expr, &values, exec_env) {
                Ok(value) => output.push((&value).into()),
                Err(e) => output.push(Output::Err(e)),
            },
            Assign(id, expr) => {
                match eval(expr, &values, exec_env) {
                    Ok(value) => {
                        values.insert(id.to_string(), value);
                    }
                    Err(e) => output.push(Output::Err(e)),
                };
            }
        }
    }
    output
}

fn eval(
    expr: &Expression,
    values: &HashMap<String, Value>,
    exec_env: &impl Env,
) -> Result<Value, Error> {
    use Expression::*;
    let value = match expr {
        Identifier(id) => values.get(id).ok_or(Error::NullDeref)?.clone(),
        StringLiteral(s) => Value::String(s.clone()),
        TransferFunction(num, den) => {
            let num = DVector::from_vec(num.clone());
            let den = DVector::from_vec(den.clone());
            let tf = DiscreteTransferFunction::new(num, den)
                .ok_or(Error::Other("Could not construct tf".to_string()))?;
            Value::TransferFunction(tf)
        }
        Tf2Ss(tf) => {
            let Value::TransferFunction(tf) = eval(tf, values, exec_env)? else {
                return Err(Error::TypeError);
            };
            let ss = tf
                .convert_to_state_space()
                .ok_or(Error::Other("Could not convert to state space".to_string()))?;
            Value::StateSpaceModel(ss)
        }
        Load(file_name) => {
            let Value::String(file_name) = eval(file_name, values, exec_env)? else {
                return Err(Error::TypeError);
            };
            let text = exec_env
                .read_file(&file_name)
                .ok_or(Error::Other(format!("file {file_name} could not be read")))?;
            Value::String(text)
        }
        Step(ss) => {
            let ss = match eval(ss, values, exec_env)? {
                Value::TransferFunction(tf) => tf
                    .convert_to_state_space()
                    .ok_or(Error::Other("Could not convert to state space".to_string()))?,
                Value::StateSpaceModel(ss) => ss,
                _ => return Err(Error::TypeError),
            };
            let output = ss.step();
            Value::Matrix(output)
        }
    };
    Ok(value)
}
