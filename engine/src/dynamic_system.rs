use log::info;
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
use std::rc::Rc;

use nalgebra::{DVector, RowDVector, Vector1};

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
    state_size: usize,
    signals_size: usize,
}

#[derive(Clone, Debug)]
struct SimulationBlock {
    executable: Rc<DiscreteStateSpaceModel>,
    state_mapping: Range<usize>,
    reads_input_from: usize,
}

#[derive(Clone, Copy, Debug)]
enum ExecutionStep {
    CalculateOutput {
        system_id: usize,
        output_position: usize,
    },
    UpdateState {
        system_id: usize,
    },
}

impl Simulation {
    pub fn new(system: &CompoundSystem) -> Option<Self> {
        let num_signals = system.components.len() + 1;

        let mut blocks = vec![];
        let mut state_size = 0;
        let mut dependencies: Vec<Vec<usize>> = vec![];
        dependencies.resize(num_signals, vec![]);

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
            let sub_system_state_count = executable.state_size();
            let state_mapping = (state_size)..(state_size + sub_system_state_count);

            if executable.has_feedthrough() {
                dependencies[i].push(component.reads_input_from);
            }
            blocks.push(SimulationBlock {
                executable,
                state_mapping,
                reads_input_from: component.reads_input_from,
            });

            state_size += sub_system_state_count;
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
            execution_plan.push(ExecutionStep::CalculateOutput {
                system_id: i,
                output_position: i + 1,
            });
        }
        for (i, block) in blocks.iter().enumerate() {
            if block.executable.state_size() > 0 {
                execution_plan.push(ExecutionStep::UpdateState { system_id: i });
            }
        }

        Some(Self {
            blocks,
            state_size,
            signals_size: num_signals,
            execution_plan,
        })
    }

    pub fn execute(&self) -> RowDVector<f64> {
        info!("{self:?}");
        let mut states = DVector::zeros(self.state_size);
        let steps = 35;
        let mut output = RowDVector::zeros(steps + 1);

        let u = 1.0;
        let mut signals = DVector::zeros(self.signals_size);
        for i in 0..=steps {
            signals[0] = u;
            for step in &self.execution_plan {
                match step {
                    ExecutionStep::CalculateOutput {
                        system_id,
                        output_position,
                    } => {
                        let block = &self.blocks[*system_id];
                        block.executable.calculate_output(
                            signals[block.reads_input_from],
                            states.rows(block.state_mapping.start, block.state_mapping.len()),
                            &mut signals[*output_position],
                        );
                    }
                    ExecutionStep::UpdateState { system_id } => {
                        let block = &self.blocks[*system_id];
                        block.executable.update_state(
                            signals[block.reads_input_from],
                            states.rows_mut(block.state_mapping.start, block.state_mapping.len()),
                        );
                    }
                };
            }
            let output_this_cycle = signals[signals.len() - 1];
            output.set_column(i, &Vector1::new(output_this_cycle));
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
