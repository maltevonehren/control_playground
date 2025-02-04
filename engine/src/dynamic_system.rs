use log::info;
use ndarray::{prelude::*, Slice};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::state_space::DiscreteStateSpaceModel;
use crate::transfer_function::DiscreteTransferFunction;

#[derive(Clone, Debug, PartialEq)]
pub enum SystemBlock {
    StateSpace(Rc<DiscreteStateSpaceModel>),
    TransferFunction(Rc<DiscreteTransferFunction>),
    Difference,
    // SubSystem(Rc<CompoundDiscreteSystem>),
}

impl fmt::Display for SystemBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemBlock::StateSpace(ss) => ss.fmt(f),
            SystemBlock::TransferFunction(tf) => tf.fmt(f),
            SystemBlock::Difference => f.write_str("−"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Simulation {
    blocks: Vec<SimulationBlock>,
    execution_plan: Vec<ExecutionStep>,
    input_signal_mapping: Slice,
    output_signal_mapping: usize,
    state_size: usize,
    signals_size: usize,
}

#[derive(Clone, Debug)]
struct SimulationBlock {
    executable: Rc<DiscreteStateSpaceModel>,
    input_signal_mapping: Slice,
    state_mapping: Slice,
    output_signal_mapping: Slice,
}

#[derive(Clone, Copy, Debug)]
enum ExecutionStep {
    CalculateOutput { system_id: usize },
    CalculateOutputWithFeedthrough { system_id: usize },
    UpdateState { system_id: usize },
}

impl Simulation {
    pub fn new(system: &CompoundSystem) -> Option<Self> {
        let mut signals_size = 0;
        let mut blocks = vec![];

        let mut state_size = 0;
        let mut dependencies: Vec<Vec<Signal>> = vec![];
        dependencies.resize(system.components.len() + 1, vec![]);
        let input_signal_mapping = (signals_size..signals_size + 1).into();
        signals_size += 1;

        // build execution graph
        // for now: calculate all signals first, then update discrete states.
        // Can be optimized later to use less intermediate memory.

        for (i, component) in system.components.iter().enumerate() {
            let executable = match &component.block {
                SystemBlock::StateSpace(ss) => ss.clone(),
                SystemBlock::TransferFunction(tf) => {
                    let b = tf.convert_to_state_space()?;
                    Rc::new(b)
                }
                SystemBlock::Difference => Rc::new(DiscreteStateSpaceModel::new(
                    Array2::zeros((0, 0)),
                    Array2::zeros((0, 2)),
                    Array2::zeros((1, 0)),
                    array![[1.0, -1.0]],
                )),
            };
            let state_mapping = (state_size..(state_size + executable.state_size())).into();
            let output_signal_mapping =
                (signals_size..(signals_size + executable.output_size())).into();
            state_size += executable.state_size();
            signals_size += executable.output_size();

            if executable.has_feedthrough() {
                for input in component.reads_input_from.iter() {
                    dependencies[i].push(*input);
                }
            }
            blocks.push(SimulationBlock {
                executable,
                input_signal_mapping: (0..0).into(), // mapped later
                state_mapping,
                output_signal_mapping,
            });
        }

        // adjust reads_input_from after all output signal have been mapped
        for (i, component) in system.components.iter().enumerate() {
            let input_mapping = match component.reads_input_from[..] {
                [input] => match input {
                    Signal::SystemInput => input_signal_mapping,
                    Signal::ComponentOutput(i) => blocks[i].output_signal_mapping,
                },
                [input1, input2] => {
                    // support having two inputs (both of size 1) by mapping
                    // to a slice with two elements and a large step
                    let input1 = match input1 {
                        Signal::SystemInput => input_signal_mapping,
                        Signal::ComponentOutput(i) => blocks[i].output_signal_mapping,
                    };
                    let input2 = match input2 {
                        Signal::SystemInput => input_signal_mapping,
                        Signal::ComponentOutput(i) => blocks[i].output_signal_mapping,
                    };
                    if Some(input1.start + 1) != input1.end || Some(input2.start + 1) != input2.end
                    {
                        panic!("can only subtract two signals of size 1");
                    }
                    let start = input1.start.min(input2.start);
                    let end = input1.start.max(input2.start);
                    Slice::new(start, Some(end + 1), input2.start - input1.start)
                }
                _ => panic!(),
            };
            blocks[i].input_signal_mapping = input_mapping;
            // TODO: ensure input and output do not overlap, if we have feedthrough (algebraic loop)
        }

        // TODO: topological sort
        // let work_set: Vec<usize> = dependencies
        //     .iter()
        //     .enumerate()
        //     .filter(|(_, v)| v.is_empty())
        //     .map(|(i, _)| i)
        //     .collect();
        // while let Some(next) = work_set.pop() {
        //     execution_plan.push(next);
        // }

        let mut execution_plan = vec![];
        for (i, block) in blocks.iter().enumerate() {
            if block.executable.has_feedthrough() {
                execution_plan.push(ExecutionStep::CalculateOutputWithFeedthrough { system_id: i });
            } else if block.executable.output_size() > 0 {
                execution_plan.push(ExecutionStep::CalculateOutput { system_id: i });
            }
        }
        for (i, block) in blocks.iter().enumerate() {
            if block.executable.state_size() > 0 {
                execution_plan.push(ExecutionStep::UpdateState { system_id: i });
            }
        }

        // TODO: take output to be last signal
        let output_signal_mapping = signals_size - 1;

        Some(Self {
            blocks,
            state_size,
            input_signal_mapping,
            output_signal_mapping,
            signals_size,
            execution_plan,
        })
    }

    pub fn execute(&self) -> Array1<f64> {
        info!("{self:?}");
        let mut states = Array1::zeros(self.state_size);
        let steps = 35;
        let mut output = Array1::zeros(steps + 1);

        let u = Array1::from_elem((1,), 1.0);
        let mut signals = Array1::zeros(self.signals_size);
        for i in 0..=steps {
            signals.slice_mut(s![self.input_signal_mapping]).assign(&u);
            for step in &self.execution_plan {
                match step {
                    ExecutionStep::CalculateOutput { system_id } => {
                        let block = &self.blocks[*system_id];
                        block.executable.calculate_output(
                            states.slice(s![block.state_mapping]),
                            signals.slice_mut(s![block.output_signal_mapping]),
                        );
                    }
                    ExecutionStep::CalculateOutputWithFeedthrough { system_id } => {
                        let block = &self.blocks[*system_id];
                        let (input, output) = signals.multi_slice_mut((
                            s![block.input_signal_mapping],
                            s![block.output_signal_mapping],
                        ));
                        block.executable.calculate_output_with_feedthrough(
                            input.view(),
                            states.slice(s![block.state_mapping]),
                            output,
                        );
                    }
                    ExecutionStep::UpdateState { system_id } => {
                        let block = &self.blocks[*system_id];
                        block.executable.update_state(
                            signals.slice(s![block.input_signal_mapping]),
                            states.slice_mut(s![block.state_mapping]),
                        );
                    }
                };
            }
            let output_this_cycle = signals[self.output_signal_mapping];
            output[i] = output_this_cycle;
        }
        output
    }
}

/// A system consisting of multiple subsystems
#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSystem {
    pub components: Vec<CompoundSystemComponent>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSystemComponent {
    pub block: SystemBlock,
    pub name: Rc<str>,
    pub reads_input_from: Rc<[Signal]>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Signal {
    SystemInput,
    ComponentOutput(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSystemComponentDefinition {
    pub block: SystemBlock,
    pub name: Rc<str>,
    pub reads_input_from: Rc<[Rc<str>]>,
}

impl CompoundSystem {
    pub fn new(components: Vec<CompoundSystemComponentDefinition>) -> Result<Self, Rc<str>> {
        // do name resolution
        let mut signal_names = HashMap::new();
        signal_names.insert("u".into(), Signal::SystemInput);
        for (i, sub_system) in components.iter().enumerate() {
            if signal_names.contains_key(&sub_system.name) {
                return Err(format!("duplicate name {}", sub_system.name).into());
            }
            signal_names.insert(sub_system.name.clone(), Signal::ComponentOutput(i));
        }

        let components = components
            .into_iter()
            .map(|c| {
                Ok(CompoundSystemComponent {
                    block: c.block,
                    name: c.name,
                    reads_input_from: c
                        .reads_input_from
                        .iter()
                        .map(|input| {
                            signal_names
                                .get(input)
                                .ok_or(format!("signal {} does not exist", input))
                                .copied()
                        })
                        .collect::<Result<_, String>>()?,
                })
            })
            .collect::<Result<_, String>>()?;

        Ok(Self { components })
    }
}
