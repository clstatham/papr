use rustc_hash::FxHashMap;

use crate::prelude::*;

use super::{
    AudioConnection, Device, OwnedDevice, MAX_AUDIO_INPUTS_PER_DEVICE, MAX_AUDIO_OUTPUTS_PER_DEVICE,
};

pub struct DeviceGraph {
    audio_graph: DiGraph<OwnedDevice, AudioConnection>,
    cached_outputs: FxHashMap<NodeIndex, [f64; MAX_AUDIO_OUTPUTS_PER_DEVICE]>,
}

impl DeviceGraph {
    pub fn new() -> Self {
        Self {
            audio_graph: DiGraph::new(),
            cached_outputs: FxHashMap::default(),
        }
    }

    pub fn add_device<T>(&mut self, name: &'static str, dev: T) -> NodeIndex
    where
        T: Device + Send + 'static,
    {
        let dev = OwnedDevice {
            name: name.to_owned(),
            dev: Box::new(dev),
            id: NodeIndex::default(),
        };
        let id = self.audio_graph.add_node(dev);
        self.audio_graph[id].id = id;
        id
    }

    pub fn connect(
        &mut self,
        source: NodeIndex,
        source_out_idx: usize,
        sink: NodeIndex,
        sink_in_idx: usize,
        amp: f64,
    ) -> EdgeIndex {
        let con = AudioConnection {
            id: EdgeIndex::default(),
            source,
            source_out_idx,
            sink,
            sink_in_idx,
            amp,
        };
        let id = self.audio_graph.add_edge(source, sink, con);
        self.audio_graph[id].id = id;
        id
    }
}

impl Default for DeviceGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for DeviceGraph {
    // TODO: actually use control inputs/outputs
    fn process(
        &mut self,
        sample_clock: f64,
        sample_rate: f64,
        audio_inputs: &[f64],
        audio_outputs: &mut [f64],
        control_inputs: &[f64],
        control_outputs: &mut [f64],
    ) {
        self.cached_outputs.clear();

        let mut inp_channel_cursor = 0;

        let mut outs = [0.; MAX_AUDIO_OUTPUTS_PER_DEVICE];

        for node_idx in self.audio_graph.node_indices() {
            if self
                .audio_graph
                .edges_directed(node_idx, Direction::Incoming)
                .count()
                > 0
            {
                continue;
            }
            outs.fill(0.);
            let n_inputs = self.audio_graph[node_idx].n_audio_inputs();
            self.audio_graph[node_idx].process(
                sample_clock,
                sample_rate,
                &audio_inputs[inp_channel_cursor..n_inputs],
                &mut outs,
                &[],
                &mut [],
            );
            self.cached_outputs.insert(node_idx, outs);

            inp_channel_cursor += n_inputs;
        }

        let mut ins = [0.; MAX_AUDIO_INPUTS_PER_DEVICE];

        while self.cached_outputs.len() < self.audio_graph.node_count() {
            'node_iter: for node_idx in self.audio_graph.node_indices() {
                if self.cached_outputs.contains_key(&node_idx) {
                    continue 'node_iter;
                }
                ins.fill(0.);
                outs.fill(0.);

                for input_con in self
                    .audio_graph
                    .edges_directed(node_idx, Direction::Incoming)
                {
                    if let Some(cached_out) = self.cached_outputs.get(&input_con.source()) {
                        ins[input_con.weight().sink_in_idx] =
                            cached_out[input_con.weight().source_out_idx] * input_con.weight().amp;
                    } else {
                        continue 'node_iter;
                    }
                }

                self.audio_graph[node_idx].process(
                    sample_clock,
                    sample_rate,
                    &ins,
                    &mut outs,
                    &[],
                    &mut [],
                );
                self.cached_outputs.insert(node_idx, outs);
            }
        }

        let mut out_channel_cursor = 0;

        for node_idx in self.audio_graph.node_indices() {
            if self
                .audio_graph
                .edges_directed(node_idx, Direction::Outgoing)
                .count()
                > 0
            {
                continue;
            }

            let n_outputs = self.audio_graph[node_idx].n_audio_outputs();

            audio_outputs[out_channel_cursor..n_outputs]
                .copy_from_slice(&self.cached_outputs[&node_idx][..n_outputs]);

            out_channel_cursor += n_outputs;
        }

        control_outputs.copy_from_slice(control_inputs);
    }

    fn n_audio_inputs(&self) -> usize {
        self.audio_graph
            .node_indices()
            .filter_map(|node| {
                if self
                    .audio_graph
                    .edges_directed(node, Direction::Incoming)
                    .count()
                    == 0
                {
                    Some(self.audio_graph[node].n_audio_inputs())
                } else {
                    None
                }
            })
            .sum()
    }

    fn n_audio_outputs(&self) -> usize {
        self.audio_graph
            .node_indices()
            .filter_map(|node| {
                if self
                    .audio_graph
                    .edges_directed(node, Direction::Outgoing)
                    .count()
                    == 0
                {
                    Some(self.audio_graph[node].n_audio_outputs())
                } else {
                    None
                }
            })
            .sum()
    }
}
