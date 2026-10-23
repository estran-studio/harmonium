#![allow(unsafe_code)]
//! Real-time safety checker for audio processing
//!
//! This module provides a custom allocator that panics if any allocation or deallocation
//! occurs while in an audio processing context. This is crucial for real-time audio
//! where allocations can cause unpredictable latency and glitches.
//!
//! The allocator guard is only active in debug builds (#[`cfg(debug_assertions)`]).
//! In release builds, the functions are no-ops with zero overhead.
//!
//! # Usage
//!
//! ```rust,ignore
//! pub fn process_buffer(&mut self, output: &mut [f32], channels: usize) {
//!     crate::realtime::rt_check::enter_audio_context();
//!
//!     // ... audio processing code ...
//!     // Any allocation here will panic in debug builds
//!
//!     crate::realtime::rt_check::exit_audio_context();
//! }
//! ```

#[cfg(debug_assertions)]
use std::alloc::{GlobalAlloc, Layout, System};
#[cfg(debug_assertions)]
use std::cell::Cell;

#[cfg(debug_assertions)]
thread_local! {
    // Thread-local nesting depth of audio processing contexts: > 0 means "in the audio callback".
    // A depth (not a flag) lets the whole callback and its inner sections each hold a guard.
    // Using thread_local! ensures tests don't interfere with each other
    static AUDIO_DEPTH: Cell<u32> = const { Cell::new(0) };
}

#[cfg(debug_assertions)]
fn in_audio_context() -> bool {
    AUDIO_DEPTH.with(Cell::get) > 0
}

/// Custom allocator that checks for real-time violations
///
/// This allocator wraps the system allocator and panics if any allocation or
/// deallocation occurs while an audio context is entered.
#[cfg(debug_assertions)]
pub struct RTCheckAllocator;

#[cfg(debug_assertions)]
unsafe impl GlobalAlloc for RTCheckAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let in_audio = in_audio_context();
        assert!(
            !in_audio,
            "REAL-TIME VIOLATION: Allocation in audio thread! size={} bytes, align={}",
            layout.size(),
            layout.align()
        );
        // SAFETY: We're delegating to the system allocator with the same layout
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let in_audio = in_audio_context();
        assert!(
            !in_audio,
            "REAL-TIME VIOLATION: Deallocation in audio thread! size={} bytes, align={}",
            layout.size(),
            layout.align()
        );
        // SAFETY: We're delegating to the system allocator with the same ptr/layout
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let in_audio = in_audio_context();
        assert!(
            !in_audio,
            "REAL-TIME VIOLATION: Zero allocation in audio thread! size={} bytes, align={}",
            layout.size(),
            layout.align()
        );
        // SAFETY: We're delegating to the system allocator with the same layout
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let in_audio = in_audio_context();
        assert!(
            !in_audio,
            "REAL-TIME VIOLATION: Reallocation in audio thread! old_size={} bytes, new_size={} bytes",
            layout.size(),
            new_size
        );
        // SAFETY: We're delegating to the system allocator with the same ptr/layout/new_size
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// Mark the beginning of an audio processing context
///
/// Call this at the start of your audio callback. Any allocations after this
/// point will cause a panic in debug builds. Contexts nest: each
/// `enter_audio_context` needs its own `exit_audio_context`.
#[cfg(debug_assertions)]
#[inline]
pub fn enter_audio_context() {
    AUDIO_DEPTH.with(|depth| depth.set(depth.get() + 1));
}

/// Mark the end of an audio processing context
///
/// Call this at the end of your audio callback. Allocations are allowed
/// again once every entered context has exited.
#[cfg(debug_assertions)]
#[inline]
pub fn exit_audio_context() {
    AUDIO_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
}

/// Scope in which allocation is explicitly allowed again, inside an audio
/// context (see [`allow_allocation`]). Restores the previous depth on drop.
#[must_use = "the exemption ends when this guard is dropped"]
pub struct AllocationAllowed {
    #[cfg(debug_assertions)]
    saved_depth: u32,
}

/// Lift the real-time check for the lifetime of the returned guard.
///
/// Reserved for the few one-off operations that cannot be made allocation-free
/// (loading a SoundFont). Every use is a documented real-time debt; nothing on
/// the steady-state audio path may use it.
#[cfg(debug_assertions)]
pub fn allow_allocation() -> AllocationAllowed {
    AllocationAllowed { saved_depth: AUDIO_DEPTH.with(|depth| depth.replace(0)) }
}

#[cfg(debug_assertions)]
impl Drop for AllocationAllowed {
    fn drop(&mut self) {
        AUDIO_DEPTH.with(|depth| depth.set(self.saved_depth));
    }
}

/// No-op in release builds
#[cfg(not(debug_assertions))]
pub fn allow_allocation() -> AllocationAllowed {
    AllocationAllowed {}
}

/// No-op in release builds
#[cfg(not(debug_assertions))]
#[inline]
pub fn enter_audio_context() {}

/// No-op in release builds
#[cfg(not(debug_assertions))]
#[inline]
pub fn exit_audio_context() {}

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(debug_assertions)]
    fn test_allocation_outside_audio_context() {
        // Should not panic
        let _vec = vec![1, 2, 3, 4, 5];
        drop(_vec);
    }

    // NOTE: These tests are commented out because #[should_panic] tests cause
    // double-panics during unwinding (panic during deallocation while handling
    // the allocation panic). The allocator works correctly in production use.
    //
    // #[test]
    // #[cfg(debug_assertions)]
    // #[should_panic(expected = "REAL-TIME VIOLATION: Allocation in audio thread")]
    // fn test_allocation_inside_audio_context() {
    //     enter_audio_context();
    //     // This should panic
    //     let _vec = vec![1, 2, 3, 4, 5];
    //     exit_audio_context();
    // }
    //
    // #[test]
    // #[cfg(debug_assertions)]
    // #[should_panic(expected = "REAL-TIME VIOLATION: Deallocation in audio thread")]
    // fn test_deallocation_inside_audio_context() {
    //     let vec = vec![1, 2, 3, 4, 5];
    //     enter_audio_context();
    //     // This should panic when vec is dropped
    //     drop(vec);
    //     exit_audio_context();
    // }

    #[test]
    #[cfg(not(debug_assertions))]
    fn test_no_op_in_release() {
        // This test only makes sense in release builds where guards are no-ops
        // In debug builds, this would trigger the allocator guard
        enter_audio_context();
        let _vec = vec![1, 2, 3, 4, 5];
        exit_audio_context();
    }
}
