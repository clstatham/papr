use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use miette::Result;

use crate::{
    dsp::Signal,
    graph::{Input, Node, NodeName, Output},
    Scalar,
};

use super::{DspError, Processor, SignalRate};

pub struct Sample {
    pub buf: Box<[Scalar]>,
}

impl Sample {
    pub fn create_node(name: &str, sample_path: PathBuf) -> Result<Arc<Node>> {
        let dec = creak::Decoder::open(sample_path).map_err(DspError::Creak)?;
        let channels = dec.info().channels();
        let buf: Box<[Scalar]> = dec
            .into_samples()
            .map_err(DspError::Creak)?
            .map(|s| s.unwrap_or(0.0) as Scalar)
            .collect::<Box<_>>()
            .chunks_exact(channels)
            .map(|ch| ch[0])
            .collect();
        Ok(Arc::new(Node::new(
            NodeName::new(name),
            vec![Input::new("seek", Some(Signal::Scalar(0.0)))],
            vec![
                Output {
                    name: "out".to_owned(),
                },
                Output {
                    name: "len".to_owned(),
                },
            ],
            crate::graph::ProcessorType::Builtin(Box::new(RwLock::new(Self { buf }))),
        )))
    }
}

impl Processor for Sample {
    fn process_sample(
        &mut self,
        _buffer_idx: usize,
        signal_rate: SignalRate,
        inputs: &[Signal],
        outputs: &mut [Signal],
    ) -> Result<()> {
        let seek = &inputs[0];
        let seek_samps = (seek.expect_scalar()? * signal_rate.rate()) as usize;

        outputs[0] = Signal::Scalar(self.buf[seek_samps]);
        outputs[1] = Signal::Scalar(self.buf.len() as Scalar / signal_rate.rate());
        Ok(())
    }
}
