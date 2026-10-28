pub mod backend;
pub mod realtime;
pub mod voice_manager;
pub mod odin_source;
#[cfg(feature = "odin")]
pub mod odin_voices;

use harmonium_core::events::AudioEvent;

use crate::backend::AudioRenderer; // Assuming backend exports this

pub struct AudioProcessor {
    renderer: Box<dyn AudioRenderer + Send>,
}

impl AudioProcessor {
    #[must_use]
    pub fn new(renderer: Box<dyn AudioRenderer + Send>) -> Self {
        Self { renderer }
    }

    pub fn process_events(&mut self, events: &[AudioEvent]) {
        for event in events {
            self.renderer.handle_event(event.clone());
        }
    }

    pub fn process_audio(&mut self, output: &mut [f32], channels: usize) {
        self.renderer.process_buffer(output, channels);
    }
}
