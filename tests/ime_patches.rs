//! Guards for the IME fixes this branch carries on top of winit 0.30.13.
//!
//! Most of these fixes can only be exercised with a real input method on the platform, so each
//! test checks that the fix is still in the source and fails if a rebase drops it. When a winit
//! release carries an equivalent fix, drop the patch together with its test.

use std::path::Path;

fn source(path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/platform_impl").join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

/// The text after `start`, up to the first `end`.
fn section<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let (_, rest) = text.split_once(start).unwrap_or_else(|| panic!("`{start}` not found"));
    rest.split_once(end).map_or(rest, |(body, _)| body)
}

#[test]
fn wayland_text_input_lives_until_the_seat_goes() {
    // winit 0.30.13 destroys a seat's text input whenever the seat loses any capability (a touch
    // screen or tablet unplugged, a remote-desktop session ending) and recreates it only when a
    // capability is added, so IME stops working until the app restarts. The text input belongs to
    // the seat instead (rust-windowing/winit#4747).
    let seat = source("linux/wayland/seat/mod.rs");
    let remove_capability = section(&seat, "fn remove_capability(", "\n    }\n");
    let remove_seat = section(&seat, "fn remove_seat(", "\n    }\n");
    assert!(
        !remove_capability.contains("text_input.destroy()"),
        "losing a capability destroys the seat's text input"
    );
    assert!(
        remove_seat.contains("text_input.destroy()"),
        "removing the seat doesn't destroy its text input"
    );
}

#[test]
fn windows_ime_cursor_is_read_only_when_reported() {
    // winit 0.30.13 reads `GCS_CURSORPOS` for every composition without a target clause, whether
    // or not `WM_IME_COMPOSITION` says the IME set it. The Microsoft Korean IME never sets it, so
    // the cursor read as 0, before the syllable being composed. Without the flag the cursor is at
    // the end of the composition (rust-windowing/winit#4746).
    let ime = source("windows/ime.rs");
    assert!(
        ime.contains("if composition_flags & GCS_CURSORPOS != 0 {"),
        "GCS_CURSORPOS is read unconditionally"
    );
    let event_loop = source("windows/event_loop.rs");
    assert!(
        event_loop.contains("get_composing_text_and_cursor(lparam as u32)"),
        "WM_IME_COMPOSITION doesn't pass its flags on"
    );
}
