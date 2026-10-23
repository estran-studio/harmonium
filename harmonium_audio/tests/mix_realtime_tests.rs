//! Application du mix et DSP de bus sous RTCheckAllocator, après préparation du synthé.
use harmonium_audio::{
    backend::{AudioRenderer, synth_backend::SynthBackend},
    realtime::rt_check,
};
use harmonium_core::{events::AudioEvent, mix::ProfileMix};

#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: rt_check::RTCheckAllocator = rt_check::RTCheckAllocator;

#[test]
fn prepared_mix_and_stereo_biquads_are_allocation_free_and_master_is_linear() {
    let font = include_bytes!("../../assets/test.sf2");
    let mut a = SynthBackend::new(44100.0, Some(font), None, &[]);
    let mut b = SynthBackend::new(44100.0, Some(font), None, &[]);
    let mut profile = ProfileMix::default();
    profile.eq.highpass_hz = 80.0;
    profile.eq.presence_hz = 2500.0;
    profile.eq.presence_db = 3.0;
    profile.room.level = 0.4;
    profile.tracks.melody.reverb_send = 0.5;
    let first = profile.prepare(44100.0);
    profile.master_db = 20.0 * 2.0_f32.log10();
    let second = profile.prepare(44100.0);
    for synth in [&mut a, &mut b] {
        synth.handle_event(AudioEvent::ProgramChange { channel: 0, program: 0 });
        synth.handle_event(AudioEvent::NoteOn { channel: 0, note: 60, velocity: 100 });
    }
    let mut left = [0.0; 8192];
    let mut right = [0.0; 8192];
    rt_check::enter_audio_context();
    a.handle_event(AudioEvent::SetMix { mix: first });
    b.handle_event(AudioEvent::SetMix { mix: second });
    a.process_buffer(&mut left, 2);
    b.process_buffer(&mut right, 2);
    rt_check::exit_audio_context();
    let energy: f32 = left.iter().map(|x| x * x).sum();
    assert!(energy > 1e-6, "signal réel de la SoundFont");
    for (unity, doubled) in left.into_iter().zip(right) {
        assert!(unity.is_finite() && doubled.is_finite());
        assert!((doubled - 2.0 * unity).abs() < 1e-6, "master +6.02 dB = amplitude ×2");
    }
}
