use egui::{
    Event, Key, Modifiers,
    text::{CCursor, CCursorRange},
};

use super::reference_editor::FrameObservation;
use super::{cursor, observation};

const WORD_TEXT: &str = "alpha beta gamma";
const WORD_CURSOR: usize = 8;
const WORD_LEFT: usize = 6;
const WORD_RIGHT: usize = 10;
const LINE_TEXT: &str = "one\nmiddle line\nthree";
const LINE_CURSOR: usize = 8;
const LINE_START: usize = 4;
const LINE_END: usize = 15;
const DOCUMENT_END: usize = 21;

fn key_event(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    }
}

fn moved(
    text: &str,
    index: usize,
    key: Key,
    modifiers: Modifiers,
) -> Result<FrameObservation, Box<dyn std::error::Error>> {
    observation(text, cursor(index), Some(key_event(key, modifiers)), true)
}

fn alt(shift: bool) -> Modifiers {
    Modifiers {
        alt: true,
        shift,
        ..Modifiers::NONE
    }
}

#[cfg(target_os = "macos")]
fn mac_command(shift: bool) -> Modifiers {
    Modifiers {
        mac_cmd: true,
        shift,
        ..Modifiers::NONE
    }
}

fn command(shift: bool) -> Modifiers {
    Modifiers {
        command: true,
        shift,
        ..Modifiers::NONE
    }
}

fn selection(primary: usize, secondary: usize) -> CCursorRange {
    CCursorRange {
        primary: cursor(primary).primary,
        secondary: cursor(secondary).primary,
        h_pos: None,
    }
}

fn alt_selection(primary: usize, secondary: usize) -> CCursorRange {
    CCursorRange {
        primary: cursor(primary).primary,
        secondary: CCursor {
            index: cursor(secondary).primary.index,
            prefer_next_row: true,
        },
        h_pos: None,
    }
}

fn assert_frame(frame: &FrameObservation, text: &str, expected: CCursorRange) {
    assert_eq!(frame.text, text);
    assert_eq!(frame.cursor, expected);
    assert!(!frame.changed);
}

#[test]
fn alt_arrows_use_word_boundaries_and_differ_from_plain_arrows()
-> Result<(), Box<dyn std::error::Error>> {
    let left = moved(WORD_TEXT, WORD_CURSOR, Key::ArrowLeft, alt(false))?;
    assert_frame(&left, WORD_TEXT, cursor(WORD_LEFT));

    let right = moved(WORD_TEXT, WORD_CURSOR, Key::ArrowRight, alt(false))?;
    assert_frame(&right, WORD_TEXT, cursor(WORD_RIGHT));

    let plain = moved(WORD_TEXT, WORD_CURSOR, Key::ArrowLeft, Modifiers::NONE)?;
    assert_frame(&plain, WORD_TEXT, cursor(WORD_CURSOR - 1));
    assert_ne!(left.cursor, plain.cursor);
    Ok(())
}

#[test]
fn alt_shift_arrows_keep_the_anchor() -> Result<(), Box<dyn std::error::Error>> {
    let left = moved(WORD_TEXT, WORD_CURSOR, Key::ArrowLeft, alt(true))?;
    assert_frame(&left, WORD_TEXT, alt_selection(WORD_LEFT, WORD_CURSOR));

    let right = moved(WORD_TEXT, WORD_CURSOR, Key::ArrowRight, alt(true))?;
    assert_frame(&right, WORD_TEXT, alt_selection(WORD_RIGHT, WORD_CURSOR));
    Ok(())
}

#[cfg(target_os = "macos")]
#[test]
fn mac_command_arrows_use_line_boundaries_and_plain_arrows_do_not()
-> Result<(), Box<dyn std::error::Error>> {
    let left = moved(LINE_TEXT, LINE_CURSOR, Key::ArrowLeft, mac_command(false))?;
    assert_frame(&left, LINE_TEXT, cursor(LINE_START));

    let right = moved(LINE_TEXT, LINE_CURSOR, Key::ArrowRight, mac_command(false))?;
    assert_frame(&right, LINE_TEXT, cursor(LINE_END));

    let plain = moved(LINE_TEXT, LINE_CURSOR, Key::ArrowLeft, Modifiers::NONE)?;
    assert_frame(&plain, LINE_TEXT, cursor(LINE_CURSOR - 1));
    assert_ne!(left.cursor, plain.cursor);
    Ok(())
}

#[cfg(target_os = "macos")]
#[test]
fn mac_command_shift_arrows_keep_the_anchor() -> Result<(), Box<dyn std::error::Error>> {
    let left = moved(LINE_TEXT, LINE_CURSOR, Key::ArrowLeft, mac_command(true))?;
    assert_frame(&left, LINE_TEXT, selection(LINE_START, LINE_CURSOR));

    let right = moved(LINE_TEXT, LINE_CURSOR, Key::ArrowRight, mac_command(true))?;
    assert_frame(&right, LINE_TEXT, selection(LINE_END, LINE_CURSOR));
    Ok(())
}

#[test]
fn command_vertical_arrows_use_document_boundaries_and_plain_arrows_do_not()
-> Result<(), Box<dyn std::error::Error>> {
    let up = moved(LINE_TEXT, LINE_CURSOR, Key::ArrowUp, command(false))?;
    assert_frame(&up, LINE_TEXT, cursor(0));

    let down = moved(LINE_TEXT, LINE_CURSOR, Key::ArrowDown, command(false))?;
    assert_frame(&down, LINE_TEXT, cursor(DOCUMENT_END));

    let plain = moved(LINE_TEXT, LINE_CURSOR, Key::ArrowUp, Modifiers::NONE)?;
    assert_ne!(up.cursor, plain.cursor);
    Ok(())
}

#[test]
fn command_shift_vertical_arrows_keep_the_anchor() -> Result<(), Box<dyn std::error::Error>> {
    let up = moved(LINE_TEXT, LINE_CURSOR, Key::ArrowUp, command(true))?;
    assert_frame(&up, LINE_TEXT, selection(0, LINE_CURSOR));

    let down = moved(LINE_TEXT, LINE_CURSOR, Key::ArrowDown, command(true))?;
    assert_frame(&down, LINE_TEXT, selection(DOCUMENT_END, LINE_CURSOR));
    Ok(())
}
