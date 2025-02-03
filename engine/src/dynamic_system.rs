use log::info;
use ndarray::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
use std::rc::Rc;

use crate::state_space::DiscreteStateSpaceModel;
use crate::transfer_function::DiscreteTransferFunction;

#[derive(Clone, Debug, PartialEq)]
pub enum SystemBlock {
    StateSpace(Rc<DiscreteStateSpaceModel>),
    TransferFunction(Rc<DiscreteTransferFunction>),
    // SubSystem(Rc<CompoundDiscreteSystem>),
}

impl fmt::Display for SystemBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemBlock::StateSpace(ss) => ss.fmt(f),
            SystemBlock::TransferFunction(tf) => tf.fmt(f),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Simulation {
    blocks: Vec<SimulationBlock>,
    execution_plan: Vec<ExecutionStep>,
    input_signal_mapping: Range<usize>,
    output_signal_mapping: usize,
    state_size: usize,
    signals_size: usize,
}

#[derive(Clone, Debug)]
struct SimulationBlock {
    executable: Rc<DiscreteStateSpaceModel>,
    input_signal_mapping: Range<usize>,
    state_mapping: Range<usize>,
    output_signal_mapping: Range<usize>,
}

#[derive(Clone, Copy, Debug)]
enum ExecutionStep {
    CalculateOutput { system_id: usize },
    UpdateState { system_id: usize },
}

impl Simulation {
    pub fn new(system: &CompoundSystem) -> Option<Self> {
        let mut signals_size = 0;
        let mut blocks = vec![];

        let mut state_size = 0;
        let mut dependencies: Vec<Vec<usize>> = vec![];
        dependencies.resize(system.components.len() + 1, vec![]);
        let input_signal_mapping = signals_size..signals_size + 1;
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
            };
            let state_mapping = (state_size)..(state_size + executable.state_size());
            let output_signal_mapping = (signals_size)..(signals_size + executable.output_size());
            state_size += executable.state_size();
            signals_size += executable.output_size();

            if executable.has_feedthrough() {
                dependencies[i].push(component.reads_input_from);
            }
            blocks.push(SimulationBlock {
                executable,
                input_signal_mapping: 0..0, // mapped later
                state_mapping,
                output_signal_mapping,
            });
        }

        // adjust reads_input_from after all output signal have been mapped
        for (i, component) in system.components.iter().enumerate() {
            let input_mapping = if component.reads_input_from == 0 {
                input_signal_mapping.clone()
            } else {
                blocks[component.reads_input_from - 1]
                    .output_signal_mapping
                    .clone()
            };
            blocks[i].input_signal_mapping = input_mapping;
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
        for i in 0..blocks.len() {
            execution_plan.push(ExecutionStep::CalculateOutput { system_id: i });
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
            signals
                .slice_mut(s![self.input_signal_mapping.clone()])
                .assign(&u);
            for step in &self.execution_plan {
                match step {
                    ExecutionStep::CalculateOutput { system_id } => {
                        let block = &self.blocks[*system_id];
                        let (input, output) = signals.multi_slice_mut((
                            s![block.input_signal_mapping.clone()],
                            s![block.output_signal_mapping.clone()],
                        ));
                        block.executable.calculate_output(
                            input.view(),
                            states.slice(s![block.state_mapping.clone()]),
                            output,
                        );
                    }
                    ExecutionStep::UpdateState { system_id } => {
                        let block = &self.blocks[*system_id];
                        block.executable.update_state(
                            signals.slice(s![block.input_signal_mapping.clone()]),
                            states.slice_mut(s![block.state_mapping.clone()]),
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
    pub reads_input_from: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSystemComponentDefinition {
    pub block: SystemBlock,
    pub name: Rc<str>,
    pub reads_input_from: Rc<str>,
}

impl CompoundSystem {
    pub fn new(components: Vec<CompoundSystemComponentDefinition>) -> Result<Self, Rc<str>> {
        // do name resolution
        let mut signal_names = HashMap::new();
        signal_names.insert("u".into(), 0);
        for (i, sub_system) in components.iter().enumerate() {
            if signal_names.contains_key(&sub_system.name) {
                return Err(format!("duplicate name {}", sub_system.name).into());
            }
            signal_names.insert(sub_system.name.clone(), i + 1);
        }

        let components = components
            .into_iter()
            .map(|c| {
                Ok(CompoundSystemComponent {
                    block: c.block,
                    name: c.name,
                    reads_input_from: *signal_names
                        .get(&c.reads_input_from)
                        .ok_or(format!("signal {} does not exist", &c.reads_input_from))?,
                })
            })
            .collect::<Result<_, String>>()?;

        Ok(Self { components })
    }
}
