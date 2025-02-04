use ndarray::{Array1, Array2, Axis};
use std::collections::HashMap;
use std::rc::Rc;

use engine::dynamic_system::{
    CompoundSystem, CompoundSystemComponentDefinition, Simulation, SystemBlock,
};
use engine::state_space::DiscreteStateSpaceModel;
use engine::transfer_function::DiscreteTransferFunction;

use crate::ast::{self, SystemItemRhs};
use ast::{Expression, Program, Statement};

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    IO(std::fmt::Error),
    NullDeref(Rc<str>),
    UnknownFunction(Rc<str>),
    TypeError,
    IncorrectNumberOfArguments(usize, usize),
    Other(Rc<str>),
}

impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Self::IO(value)
    }
}

/// Runtime Value
#[derive(Clone, Debug, PartialEq)]
enum Value {
    String(Rc<str>),
    Float(f64),
    Vector(Rc<Array1<f64>>),
    Matrix(Rc<Array2<f64>>),
    BuiltInFunction(BuiltInFunction),
    TransferFunction(Rc<DiscreteTransferFunction>),
    StateSpaceModel(Rc<DiscreteStateSpaceModel>),
    CompoundSystem(Rc<CompoundSystem>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    Err(Error),
    Text(Rc<str>),
    Plot(Rc<Array2<f64>>),
    System(Rc<CompoundSystem>),
}

impl From<&Value> for Output {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(s) => Output::Text(s.clone()),
            Value::Vector(data) => Output::Text(data.to_string().into()),
            Value::Matrix(data) => Output::Plot(data.clone()),
            Value::Float(f) => Output::Text(f.to_string().into()),
            Value::BuiltInFunction(_) => Output::Text("<builtin_function>".to_string().into()),
            Value::TransferFunction(tf) => Output::Text(tf.to_string().into()),
            Value::StateSpaceModel(ss) => Output::Text(ss.to_string().into()),
            Value::CompoundSystem(s) => Output::System(s.clone()),
        }
    }
}

impl Value {
    fn get_system(&self) -> Result<SystemBlock, Error> {
        match self {
            Value::TransferFunction(tf) => Ok(SystemBlock::TransferFunction(tf.clone())),
            Value::StateSpaceModel(ss) => Ok(SystemBlock::StateSpace(ss.clone())),
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

fn get_default_values() -> HashMap<Rc<str>, Value> {
    use BuiltInFunction::*;
    let mut values = HashMap::new();
    values.insert("load".into(), Value::BuiltInFunction(Load));
    values.insert("tf".into(), Value::BuiltInFunction(TransferFunction));
    values.insert("tf2ss".into(), Value::BuiltInFunction(Tf2Ss));
    values.insert("step".into(), Value::BuiltInFunction(Step));
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
                        values.insert(id.clone(), value);
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
    values: &HashMap<Rc<str>, Value>,
    exec_env: &impl Env,
) -> Result<Value, Error> {
    use Expression::*;
    let value = match expr {
        Identifier(id) => values.get(id).ok_or(Error::NullDeref(id.clone()))?.clone(),
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
            Value::Vector(Rc::new(Array1::from_vec(elements)))
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
            let Value::BuiltInFunction(function) =
                eval(function, values, exec_env).map_err(|e| match e {
                    Error::NullDeref(id) => Error::UnknownFunction(id),
                    e => e,
                })?
            else {
                return Err(Error::TypeError);
            };
            let num_args = arguments.len();
            match function {
                Load => {
                    if num_args != 1 {
                        return Err(Error::IncorrectNumberOfArguments(1, num_args));
                    }
                    let Value::String(file_name) = eval(&arguments[0], values, exec_env)? else {
                        return Err(Error::TypeError);
                    };
                    let text = exec_env.read_file(&file_name).ok_or(Error::Other(
                        format!("file {file_name} could not be read").into(),
                    ))?;
                    let mut rdr = csv::ReaderBuilder::new()
                        .has_headers(false)
                        .from_reader(text.as_bytes());
                    let mut m = Array2::zeros((0, 0));
                    for (i, result) in rdr.records().enumerate() {
                        let record =
                            result.map_err(|_| Error::Other("Error while parsing csv".into()))?;
                        if i == 0 {
                            m = Array2::zeros((record.len(), 0));
                        }
                        m.push(
                            Axis(0),
                            Array1::from_iter(record.iter().map(|v| v.parse().unwrap())).view(),
                        )
                        .expect("all columns to be of equal length");
                    }
                    Value::Matrix(Rc::new(m))
                }
                TransferFunction => {
                    if num_args != 2 {
                        return Err(Error::IncorrectNumberOfArguments(2, num_args));
                    }
                    let Value::Vector(num) = eval(&arguments[0], values, exec_env)? else {
                        return Err(Error::TypeError);
                    };
                    let Value::Vector(den) = eval(&arguments[1], values, exec_env)? else {
                        return Err(Error::TypeError);
                    };
                    let tf = DiscreteTransferFunction::new((*num).clone(), (*den).clone())
                        .ok_or(Error::Other("Could not construct tf".into()))?;
                    Value::TransferFunction(Rc::new(tf))
                }
                Tf2Ss => {
                    if num_args != 1 {
                        return Err(Error::IncorrectNumberOfArguments(1, num_args));
                    }
                    let Value::TransferFunction(tf) = eval(&arguments[0], values, exec_env)? else {
                        return Err(Error::TypeError);
                    };
                    let ss = tf
                        .convert_to_state_space()
                        .ok_or(Error::Other("Could not convert to state space".into()))?;
                    Value::StateSpaceModel(Rc::new(ss))
                }
                Step => {
                    if num_args != 1 {
                        return Err(Error::IncorrectNumberOfArguments(1, num_args));
                    }
                    let system = eval(&arguments[0], values, exec_env)?;
                    let system = match system {
                        Value::CompoundSystem(s) => s,
                        other => {
                            let Ok(block) = other.get_system() else {
                                return Err(Error::TypeError);
                            };
                            CompoundSystem::new(vec![CompoundSystemComponentDefinition {
                                block,
                                name: "".into(),
                                reads_input_from: ["u".into()].into(),
                            }])
                            .map_err(Error::Other)?
                            .into()
                        }
                    };
                    let sim = Simulation::new(&system)
                        .ok_or(Error::Other("could not init sim".into()))?;
                    let output = sim.execute();
                    Value::Matrix(Rc::new(output.insert_axis(Axis(0))))
                }
            }
        }
        System(items) => {
            let mut sub_systems = Vec::new();
            for item in items {
                let (inputs, system): (Rc<[Rc<str>]>, SystemBlock) = match &item.rhs {
                    SystemItemRhs::Difference {
                        input1_name,
                        input2_name,
                    } => (
                        [input1_name.clone(), input2_name.clone()].into(),
                        SystemBlock::Difference,
                    ),
                    SystemItemRhs::System {
                        system_name,
                        input_name,
                    } => (
                        [input_name.clone()].into(),
                        values
                            .get(system_name)
                            .ok_or(Error::NullDeref(system_name.clone()))?
                            .get_system()?,
                    ),
                };
                let inputs: Rc<[Rc<str>]> = inputs;
                sub_systems.push(CompoundSystemComponentDefinition {
                    block: system,
                    reads_input_from: inputs,
                    name: item.output_name.clone(),
                });
            }
            Value::CompoundSystem(Rc::new(
                CompoundSystem::new(sub_systems).map_err(Error::Other)?,
            ))
        }
    };
    Ok(value)
}
