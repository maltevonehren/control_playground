use std::collections::HashMap;
use std::rc::Rc;

use engine::dynamic_system::{CompoundDiscreteSystem, DiscreteSystem};
use engine::dynamic_system::{Simulation, SubSystem};
use engine::state_space::DiscreteStateSpaceModel;
use engine::transfer_function::DiscreteTransferFunction;
use nalgebra::{DMatrix, DVector};

use crate::ast;
use ast::{Expression, Program, Statement};

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
    Float(f64),
    Vector(Rc<DVector<f64>>),
    Matrix(Rc<DMatrix<f64>>),
    BuiltInFunction(BuiltInFunction),
    TransferFunction(Rc<DiscreteTransferFunction>),
    StateSpaceModel(Rc<DiscreteStateSpaceModel>),
    CompoundSystem(Rc<CompoundDiscreteSystem>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    Err(Error),
    Text(String),
    Plot(Rc<DMatrix<f64>>),
}

impl From<&Value> for Output {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(s) => Output::Text(s.clone()),
            Value::Vector(data) => Output::Text(data.to_string()),
            Value::Matrix(data) => Output::Plot(data.clone()),
            Value::Float(f) => Output::Text(f.to_string()),
            Value::BuiltInFunction(_) => Output::Text("<builtinfunction>".to_string()),
            Value::TransferFunction(tf) => Output::Text(tf.to_string()),
            Value::StateSpaceModel(ss) => Output::Text(ss.to_string()),
            Value::CompoundSystem(_) => Output::Text("<compoundsystem>".to_string()),
        }
    }
}

impl Value {
    fn get_sytem(&self) -> Result<Rc<dyn DiscreteSystem>, Error> {
        match self {
            Value::TransferFunction(tf) => {
                let ss = tf
                    .convert_to_state_space()
                    .ok_or(Error::Other("Could not convert to state space".to_string()))?;
                Ok(Rc::new(ss))
            }
            Value::StateSpaceModel(s) => Ok(s.clone()),
            _ => Err(Error::TypeError),
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
        FloatLiteral(f) => Value::Float(*f),
        VectorLiteral(elements) => {
            let elements = elements
                .iter()
                .map(|e| match eval(e, values, exec_env) {
                    Ok(Value::Float(f)) => Ok(f),
                    Ok(_) => Err(Error::TypeError),
                    Err(e) => Err(e),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Value::Vector(Rc::new(DVector::from_vec(elements)))
        }
        UnOp(op, e) => {
            use ast::UnOp::*;
            let Value::Float(f) = eval(e, values, exec_env)? else {
                return Err(Error::TypeError);
            };
            match op {
                Neg => Value::Float(-f),
            }
        }
        BinOp(op, e1, e2) => {
            use ast::BinOp::*;
            let Value::Float(f1) = eval(e1, values, exec_env)? else {
                return Err(Error::TypeError);
            };
            let Value::Float(f2) = eval(e2, values, exec_env)? else {
                return Err(Error::TypeError);
            };
            match op {
                Add => Value::Float(f1 + f2),
                Sub => Value::Float(f1 - f2),
                Mul => Value::Float(f1 * f2),
                Div => Value::Float(f1 / f2),
            }
        }
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
                    Value::Matrix(Rc::new(m))
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
                    let tf = DiscreteTransferFunction::new((*num).clone(), (*den).clone())
                        .ok_or(Error::Other("Could not construct tf".to_string()))?;
                    Value::TransferFunction(Rc::new(tf))
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
                    Value::StateSpaceModel(Rc::new(ss))
                }
                Step => {
                    if num_args != 1 {
                        return Err(Error::IncorectNumberOfArguments(1, num_args));
                    }
                    let system = eval(&arguments[0], values, exec_env)?;
                    let system = match system {
                        Value::TransferFunction(_) | Value::StateSpaceModel(_) => {
                            return Err(Error::Other("TODO".to_string()))
                        }
                        Value::CompoundSystem(s) => s,
                        _ => return Err(Error::TypeError),
                    };
                    let sim = Simulation::new(&system)
                        .ok_or(Error::Other("could not init sim".to_string()))?;
                    let output = sim.execute();
                    Value::Matrix(Rc::new(output.insert_rows(0, 0, 0.))) // TODO: is there a better way to convert to DMatrix?
                }
            }
        }
        System(items) => {
            let mut sub_systems = Vec::new();
            for item in items {
                let system = values
                    .get(&item.item_name)
                    .ok_or(Error::NullDeref)?
                    .get_sytem()?;
                sub_systems.push(SubSystem {
                    system,
                    input_name: item.input_name.clone(),
                    output_name: item.output_name.clone(),
                });
            }
            Value::CompoundSystem(Rc::new(
                CompoundDiscreteSystem::new(sub_systems).map_err(Error::Other)?,
            ))
        }
    };
    Ok(value)
}
