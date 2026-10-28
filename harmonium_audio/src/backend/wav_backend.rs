use std::{fs::File, io::BufWriter};

use harmonium_core::events::AudioEvent;
use hound::{WavSpec, WavWriter};

use crate::backend::AudioRenderer;

pub struct WavBackend {
    inner: Box<dyn AudioRenderer>,
    writer: WavWriter<BufWriter<File>>,
}

impl WavBackend {
    pub fn new(
        inner: Box<dyn AudioRenderer>,
        path: &str,
        sample_rate: u32,
    ) -> Result<Self, hound::Error> {
        let spec = WavSpec {
            channels: 2,
            sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let writer = WavWriter::create(path, spec)?;
        Ok(Self { inner, writer })
    }
}

impl AudioRenderer for WavBackend {
    fn handle_event(&mut self, event: AudioEvent) {
        self.inner.handle_event(event);
    }

    #[cfg(feature = "odin")]
    fn apply_odin(&mut self, update: &mut crate::odin_bank::OdinUpdate) {
        self.inner.apply_odin(update);
    }

    fn process_buffer(&mut self, output: &mut [f32], channels: usize) {
        self.inner.process_buffer(output, channels);
        for sample in output.iter() {
            self.writer.write_sample(*sample).ok();
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
