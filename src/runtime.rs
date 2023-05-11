use std::future::Future;

use cpal::{
    Device as CpalDevice, Host, OutputCallbackInfo, SampleFormat, Stream, SupportedStreamConfig,
};

use crate::{
    device::{
        device_impls::{Passthrough, SineOsc},
        graph::DeviceGraph,
        Device,
    },
    prelude::*,
};

// fn on_sample(o: &mut SampleRequestOptions) -> f64 {
//     o.tick();
// }

fn on_window<T>(
    output: &mut [T],
    request: &mut SampleRequestOptions,
    device_graph: &mut DeviceGraph,
    global_volume: f64,
) where
    T: Sample + FromSample<f64>,
{
    let mut buf = vec![0.; request.nchannels];
    for frame in output.chunks_mut(request.nchannels) {
        // let value: T = T::from_sample(on_sample(request) * global_volume);
        request.tick();
        device_graph.process(
            request.sample_clock,
            request.sample_rate,
            &[],
            &mut buf,
            &[],
            &mut [],
        );
        for (i, sample) in buf.iter().enumerate() {
            frame[i] = T::from_sample(*sample * global_volume);
        }
    }
}

pub struct SampleRequestOptions {
    pub sample_rate: f64,
    pub sample_clock: f64,
    pub nchannels: usize,
}
impl SampleRequestOptions {
    pub fn tick(&mut self) {
        self.sample_clock = (self.sample_clock + 1.0) % self.sample_rate;
    }
}

pub struct Runtime {
    tokio_rt: Option<tokio::runtime::Runtime>,
    #[allow(unused)]
    host: Host,
    output_device: CpalDevice,
    output_config: SupportedStreamConfig,
    stream: Option<Stream>,
    pub global_volume: f64,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let output_device = host
            .default_output_device()
            .ok_or_else(|| Error::msg("Default audio output device not available"))?;
        log::info!("Output device: {}", output_device.name()?);

        let output_config = output_device.default_output_config()?;
        log::info!("Output config: {:?}", output_config);

        let mut this = Self {
            global_volume: 0.1,
            stream: None,
            host,
            output_device,
            output_config,
            tokio_rt: Some(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?,
            ),
        };
        let mut device_graph = DeviceGraph::new();
        let dummy = device_graph.add_device("dummy", Passthrough { n_channels: 1 });
        let sine0 = device_graph.add_device("sine0", SineOsc { freq: 440. });
        device_graph.connect(sine0, 0, dummy, 0, 1.0);
        match this.output_config.sample_format() {
            SampleFormat::F32 => this.init_stream::<f32>(device_graph)?,
            SampleFormat::F64 => this.init_stream::<f64>(device_graph)?,
            SampleFormat::I8 => this.init_stream::<i8>(device_graph)?,
            SampleFormat::I16 => this.init_stream::<i16>(device_graph)?,
            SampleFormat::I32 => this.init_stream::<i32>(device_graph)?,
            SampleFormat::I64 => this.init_stream::<i64>(device_graph)?,
            SampleFormat::U8 => this.init_stream::<u8>(device_graph)?,
            SampleFormat::U16 => this.init_stream::<u16>(device_graph)?,
            SampleFormat::U32 => this.init_stream::<u32>(device_graph)?,
            SampleFormat::U64 => this.init_stream::<u64>(device_graph)?,
            format => return Err(Error::msg(format!("Unsupported sample format: {format}"))),
        }
        Ok(this)
    }

    fn init_stream<T>(&mut self, mut device_graph: DeviceGraph) -> Result<()>
    where
        T: SizedSample + FromSample<f64>,
    {
        let sample_rate = self.output_config.sample_rate().0 as f64;
        let sample_clock = 0f64;
        let nchannels = self.output_config.channels() as usize;
        let mut request = SampleRequestOptions {
            sample_rate,
            sample_clock,
            nchannels,
        };
        let err_fn = |err| log::error!("Error in audio output stream: {}", err);
        let global_volume = self.global_volume;
        let stream = self.output_device.build_output_stream(
            &self.output_config.config(),
            move |output: &mut [T], _: &OutputCallbackInfo| {
                on_window(output, &mut request, &mut device_graph, global_volume);
            },
            err_fn,
            None,
        )?;

        self.stream = Some(stream);
        Ok(())
    }

    pub const fn is_running(&self) -> bool {
        self.tokio_rt.is_none()
    }

    pub const fn can_run(&self) -> bool {
        self.tokio_rt.is_some()
    }

    pub fn run<F>(&mut self, f: F) -> Result<()>
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let rt = self.tokio_rt.take().ok_or(Error::msg("Cannot start PAPR runtime: inner Tokio runtime not present, is the PAPR runtime already running?"))?;
        let stream = self.stream.as_ref().ok_or(Error::msg(
            "Cannot start PAPR output stream: inner CPAL stream not present",
        ))?;
        stream.play()?;
        std::thread::Builder::new()
            .name("papr-runtime".into())
            .spawn(move || {
                rt.block_on(f)?;
                Ok::<(), anyhow::Error>(())
            })?
            .join()
            .unwrap()?;
        Ok(())
    }
}
