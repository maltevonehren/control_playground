use std::{fmt, rc::Rc};

use nalgebra::{DVector, DVectorView, DVectorViewMut, RowDVector, Vector1};

pub trait DiscreteSystem {
    fn num_states(&self) -> usize;
    // fn num_inputs(&self) -> usize;
    // fn num_outputs(&self) -> usize;
    // fn has_feedthrough(&self) -> usize;

    // TODO: MIMO, pass states as mut to avoid copy
    fn update_states(&self, input: f64, states: DVectorViewMut<'_, f64>);
    fn calculate_output(&self, input: f64, states: DVectorView<'_, f64>) -> f64;
}

pub fn step(system: &dyn DiscreteSystem) -> RowDVector<f64> {
    let mut states = DVector::zeros(system.num_states());
    let steps = 35;
    let mut output = RowDVector::zeros(steps + 1);

    let u = 1.0;
    for i in 0..=steps {
        // calculate y first so we can update x in place
        let y = system.calculate_output(u, states.as_view());
        system.update_states(u, states.as_view_mut());
        output.set_column(i, &Vector1::new(y));
    }
    output
}

/// A system consisting of multiple subsystems
#[derive(Clone)]
pub struct CompoundDiscreteSystem {
    /// for now: single line of sub systems where the output of system i is input of system i+1
    /// and the last one feeds back to the first one with a gain of -1.
    /// Asume the last system has no feedthrough,
    pub sub_systems: Vec<Rc<dyn DiscreteSystem>>,
}

impl fmt::Debug for CompoundDiscreteSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl PartialEq for CompoundDiscreteSystem {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl DiscreteSystem for CompoundDiscreteSystem {
    fn num_states(&self) -> usize {
        self.sub_systems
            .iter()
            .fold(0, |acc, x| acc + x.num_states())
    }

    fn update_states(&self, input: f64, mut states: DVectorViewMut<'_, f64>) {
        let mut state_index = 0;
        let mut next_input = input - self.calculate_output(input, states.as_view());
        for sub_system in &self.sub_systems {
            let num_states = sub_system.num_states();
            // calculate output first so we can update x in place
            let output =
                sub_system.calculate_output(next_input, states.rows(state_index, num_states));
            sub_system.update_states(next_input, states.rows_mut(state_index, num_states));
            next_input = output;
            state_index += num_states;
        }
    }

    fn calculate_output(&self, _input: f64, states: DVectorView<'_, f64>) -> f64 {
        assert!(!self.sub_systems.is_empty());
        let last = &self.sub_systems[self.sub_systems.len() - 1];
        let state_index = states.len() - last.num_states();
        // assume no feedthrough so input does not matter
        last.calculate_output(0.0, states.rows(state_index, last.num_states()))
    }
}
