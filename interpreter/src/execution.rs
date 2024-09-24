use std::collections::HashMap;

use engine::state_space::DiscreteStateSpaceModel;
use engine::transfer_function::DiscreteTransferFunction;
use nalgebra::{DMatrix, DVector};

use crate::ast::{Expression, Program, Statement};

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    IO(std::fmt::Error),
    NullDeref,
    TypeError,
    IncorectNumberOfArguments(usize, usize),
    Other(String),
}

impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Self::IO(value)
    }
}

/// Runtime Value
#[derive(Clone, Debug, PartialEq)]
enum Value {
    String(String),
    BuiltInFunction(BuiltInFunction),
    TransferFunction(DiscreteTransferFunction),
    StateSpaceModel(DiscreteStateSpaceModel),
    Vector(DVector<f64>),
    Matrix(DMatrix<f64>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    Err(Error),
    Text(String),
    Plot(DMatrix<f64>),
}

impl From<&Value> for Output {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(s) => Output::Text(s.clone()),
            Value::BuiltInFunction(_) => todo!(),
            Value::TransferFunction(tf) => Output::Text(tf.to_string()),
            Value::StateSpaceModel(ss) => Output::Text(ss.to_string()),
            Value::Vector(data) => Output::Text(data.to_string()),
            Value::Matrix(data) => Output::Plot(data.clone()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum BuiltInFunction {
    Load,
    TransferFunction,
    Tf2Ss,
    Step,
}

pub trait Env {
    fn read_file(&self, name: &str) -> Option<String>;
}

fn get_default_values() -> HashMap<String, Value> {
    use BuiltInFunction::*;
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("load".to_owned(), Value::BuiltInFunction(Load));
    values.insert("tf".to_owned(), Value::BuiltInFunction(TransferFunction));
    values.insert("tf2ss".to_owned(), Value::BuiltInFunction(Tf2Ss));
    values.insert("step".to_owned(), Value::BuiltInFunction(Step));
    values
}

pub fn execute(program: &Program, exec_env: &impl Env) -> Vec<Output> {
    use Statement::*;
    let mut output = Vec::new();
    let mut values = get_default_values();
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
        VectorLiteral(elements) => Value::Vector(DVector::from_vec(elements.clone())),
        FunctionCall {
            function,
            arguments,
        } => {
            use BuiltInFunction::*;
            let Value::BuiltInFunction(function) = eval(function, values, exec_env)? else {
                return Err(Error::TypeError);
            };
            let num_args = arguments.len();
            match function {
                Load => {
                    if num_args != 1 {
                        return Err(Error::IncorectNumberOfArguments(1, num_args));
                    }
                    let Value::String(file_name) = eval(&arguments[0], values, exec_env)? else {
                        return Err(Error::TypeError);
                    };
                    let text = exec_env
                        .read_file(&file_name)
                        .ok_or(Error::Other(format!("file {file_name} could not be read")))?;
                    let mut rdr = csv::ReaderBuilder::new()
                        .has_headers(false)
                        .from_reader(text.as_bytes());
                    let mut m = DMatrix::zeros(0, 0);
                    for (i, result) in rdr.records().enumerate() {
                        let record = result
                            .map_err(|_| Error::Other("Error while parsing csv".to_string()))?;
                        if i == 0 {
                            m = m.resize(record.len(), 0, 0.);
                        }
                        m = m.insert_column(i, 0.);
                        for (j, v) in record.iter().enumerate() {
                            m[(j, i)] = v.parse().unwrap();
                        }
                    }
                    Value::Matrix(m)
                }
                TransferFunction => {
                    if num_args != 2 {
                        return Err(Error::IncorectNumberOfArguments(2, num_args));
                    }
                    let Value::Vector(num) = eval(&arguments[0], values, exec_env)? else {
                        return Err(Error::TypeError);
                    };
                    let Value::Vector(den) = eval(&arguments[1], values, exec_env)? else {
                        return Err(Error::TypeError);
                    };
                    let tf = DiscreteTransferFunction::new(num, den)
                        .ok_or(Error::Other("Could not construct tf".to_string()))?;
                    Value::TransferFunction(tf)
                }
                Tf2Ss => {
                    if num_args != 1 {
                        return Err(Error::IncorectNumberOfArguments(1, num_args));
                    }
                    let Value::TransferFunction(tf) = eval(&arguments[0], values, exec_env)? else {
                        return Err(Error::TypeError);
                    };
                    let ss = tf
                        .convert_to_state_space()
                        .ok_or(Error::Other("Could not convert to state space".to_string()))?;
                    Value::StateSpaceModel(ss)
                }
                Step => {
                    if num_args != 1 {
                        return Err(Error::IncorectNumberOfArguments(1, num_args));
                    }
                    let ss = match eval(&arguments[0], values, exec_env)? {
                        Value::TransferFunction(tf) => tf
                            .convert_to_state_space()
                            .ok_or(Error::Other("Could not convert to state space".to_string()))?,
                        Value::StateSpaceModel(ss) => ss,
                        _ => return Err(Error::TypeError),
                    };
                    let output = ss.step();
                    Value::Matrix(output.insert_rows(0, 0, 0.)) // TODO: is there a better way to convert to DMatrix?
                }
            }
        }
    };
    Ok(value)
}
