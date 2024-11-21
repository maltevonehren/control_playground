use log::info;
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
use std::rc::Rc;

use nalgebra::{DVector, DVectorView, DVectorViewMut, RowDVector, Vector1};

// pub enum System {
//     Single(Rc<dyn DiscreteSystem>),
//     Coumpound(Rc<CompoundDiscreteSystem>),
// }

pub trait DiscreteSystem: std::fmt::Debug {
    fn state_size(&self) -> usize;
    // fn input_size(&self) -> usize;
    // fn output_size(&self) -> usize;
    fn has_feedthrough(&self) -> bool;

    // TODO: MIMO
    fn calculate_output(&self, input: f64, state: DVectorView<'_, f64>, output: &mut f64);
    fn update_state(&self, input: f64, state: DVectorViewMut<'_, f64>);
}

#[derive(Clone, Debug)]
pub struct Simulation {
    systems: Vec<(Rc<dyn DiscreteSystem>, Range<usize>)>,
    state_size: usize,
    signals_size: usize,
    execution_plan: Vec<ExecutionStep>,
}

#[derive(Clone, Copy, Debug)]
enum ExecutionStep {
    CalculateOutput {
        system_id: usize,
        input_position: usize,
        output_position: usize,
    },
    UpdateState {
        system_id: usize,
        input_position: usize,
    },
}

impl Simulation {
    pub fn new(system: &CompoundDiscreteSystem) -> Option<Self> {
        let mut individual_state_mapping = vec![];
        let mut state_size = 0;
        for (sub_system, _) in &system.sub_systems {
            let sub_system_state_count = sub_system.system.state_size();
            individual_state_mapping.push(state_size..state_size + sub_system_state_count);
            state_size += sub_system_state_count;
        }

        // build execution graph
        // for now: calculate all signals first, then update discrete states.
        // Can be optimized later to use less intermediate memory.

        let num_signals = system.sub_systems.len() + 1;
        // dependency graph
        let mut dependencies: Vec<Vec<usize>> = vec![];
        dependencies.resize(num_signals, vec![]);
        for (i, (sub_system, input_index)) in system.sub_systems.iter().enumerate() {
            if sub_system.system.has_feedthrough() {
                dependencies[i].push(*input_index);
            }
        }

        let mut execution_plan = vec![];
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
        for (i, (_, get_input_from)) in system.sub_systems.iter().enumerate() {
            execution_plan.push(ExecutionStep::CalculateOutput {
                system_id: i,
                input_position: *get_input_from,
                output_position: i + 1,
            });
        }
        for (i, (sub_system, get_input_from)) in system.sub_systems.iter().enumerate() {
            if sub_system.system.state_size() > 0 {
                execution_plan.push(ExecutionStep::UpdateState {
                    system_id: i,
                    input_position: *get_input_from,
                });
            }
        }

        Some(Self {
            systems: system
                .sub_systems
                .iter()
                .enumerate()
                .map(|(i, ss)| (ss.0.system.clone(), individual_state_mapping[i].clone()))
                .collect(),
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
                        input_position,
                        output_position,
                    } => {
                        let (system, state_position) = &self.systems[*system_id];
                        system.calculate_output(
                            signals[*input_position],
                            states.rows(state_position.start, state_position.len()),
                            &mut signals[*output_position],
                        );
                    }
                    ExecutionStep::UpdateState {
                        system_id,
                        input_position,
                    } => {
                        let (system, state_position) = &self.systems[*system_id];
                        system.update_state(
                            signals[*input_position],
                            states.rows_mut(state_position.start, state_position.len()),
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

#[derive(Clone)]
pub struct SubSystem {
    pub system: Rc<dyn DiscreteSystem>,
    pub input_name: String,
    pub output_name: String,
}

/// A system consisting of multiple subsystems
#[derive(Clone)]
pub struct CompoundDiscreteSystem {
    sub_systems: Vec<(SubSystem, usize)>,
}

impl fmt::Debug for CompoundDiscreteSystem {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl PartialEq for CompoundDiscreteSystem {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl CompoundDiscreteSystem {
    pub fn new(sub_systems: Vec<SubSystem>) -> Result<Self, String> {
        // do name resolution
        let mut signal_names = HashMap::new();
        signal_names.insert("u".to_string(), 0);
        for (i, sub_system) in sub_systems.iter().enumerate() {
            if signal_names.contains_key(&sub_system.output_name) {
                return Err(format!("duplicate output name {}", sub_system.output_name));
            }
            signal_names.insert(sub_system.output_name.clone(), i + 1);
        }

        let mut sub_systems_with_input = Vec::with_capacity(sub_systems.len());
        for sub_system in sub_systems {
            let gets_input_from = signal_names
                .get(&sub_system.input_name)
                .ok_or(format!("signal {} does not exist", &sub_system.input_name))?;
            sub_systems_with_input.push((sub_system.clone(), *gets_input_from));
        }

        Ok(Self {
            sub_systems: sub_systems_with_input,
        })
    }
}
