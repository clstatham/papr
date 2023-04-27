use std::fmt::Debug;

use petgraph::prelude::*;

pub mod device_impls;
pub mod graph;

pub const MAX_AUDIO_INPUTS_PER_DEVICE: usize = 64;
pub const MAX_AUDIO_OUTPUTS_PER_DEVICE: usize = 64;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct AudioConnection {
    pub id: EdgeIndex,
    pub source: NodeIndex,
    pub source_out_idx: usize,
    pub sink: NodeIndex,
    pub sink_in_idx: usize,
    pub amp: f64,
}

#[non_exhaustive]
pub struct OwnedDevice {
    pub name: String,
    pub dev: Box<dyn Device + Send>,
    pub id: NodeIndex,
}

impl OwnedDevice {}

impl Debug for OwnedDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {}", self.id, self.name)
    }
}

impl Device for OwnedDevice {
    fn n_audio_inputs(&self) -> usize {
        self.dev.n_audio_inputs()
    }

    fn n_audio_outputs(&self) -> usize {
        self.dev.n_audio_outputs()
    }

    fn process(
        &mut self,
        sample_clock: f64,
        sample_rate: f64,
        audio_inputs: &[f64],
        audio_outputs: &mut [f64],
        control_inputs: &[f64],
        control_outputs: &mut [f64],
    ) {
        self.dev.process(
            sample_clock,
            sample_rate,
            audio_inputs,
            audio_outputs,
            control_inputs,
            control_outputs,
        )
    }
}

pub trait Device {
    fn process(
        &mut self,
        sample_clock: f64,
        sample_rate: f64,
        audio_inputs: &[f64],
        audio_outputs: &mut [f64],
        control_inputs: &[f64],
        control_outputs: &mut [f64],
    );

    fn n_audio_inputs(&self) -> usize;
    fn n_audio_outputs(&self) -> usize;
}
