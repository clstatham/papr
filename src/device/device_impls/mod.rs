use std::f64::consts::PI;

use super::Device;

pub struct Passthrough {
    pub n_channels: usize,
}

impl Device for Passthrough {
    fn process(
        &mut self,
        _sample_clock: f64,
        _sample_rate: f64,
        audio_inputs: &[f64],
        audio_outputs: &mut [f64],
        control_inputs: &[f64],
        control_outputs: &mut [f64],
    ) {
        audio_outputs.copy_from_slice(audio_inputs);
        control_outputs.copy_from_slice(control_inputs);
    }

    fn n_audio_inputs(&self) -> usize {
        self.n_channels
    }

    fn n_audio_outputs(&self) -> usize {
        self.n_channels
    }
}

pub struct SineOsc {
    pub freq: f64,
}

impl Device for SineOsc {
    fn n_audio_inputs(&self) -> usize {
        0
    }

    fn n_audio_outputs(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        sample_clock: f64,
        sample_rate: f64,
        _audio_inputs: &[f64],
        audio_outputs: &mut [f64],
        _control_inputs: &[f64],
        _control_outputs: &mut [f64],
    ) {
        audio_outputs[0] = (sample_clock * self.freq * 2.0 * PI / sample_rate).sin();
    }
}
