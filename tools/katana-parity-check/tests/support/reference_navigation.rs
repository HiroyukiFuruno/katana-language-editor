use egui::{Event, Key, Modifiers, text::CCursorRange};

use super::reference_editor::{FrameObservation, ReferenceEditor};
use super::{cursor, observation};

const EDIT_ID_SOURCE: &str = "fixed-egui-reference-navigation";
const SAMPLE_TEXT: &str = "abcd";
const LINE_TEXT: &str = "abc\ndef\nghi";
const EMPTY: usize = 0;
const SAMPLE_START: usize = 1;
const SAMPLE_MIDDLE: usize = 2;
const SAMPLE_END: usize = 3;
const LINE_START: usize = 4;
const LINE_END: usize = 7;

fn event(key: Key, modifiers: Modifiers, pressed: bool) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers,
    }
}

fn key_event(key: Key, modifiers: Modifiers) -> Event {
    event(key, modifiers, true)
}

fn released_key(key: Key) -> Event {
    event(key, Modifiers::NONE, false)
}

fn modifiers(shift: bool, command: bool) -> Modifiers {
    Modifiers {
        shift,
        command,
        mac_cmd: command,
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

fn assert_frame(frame: &FrameObservation, text: &str, expected: CCursorRange, changed: bool) {
    assert_eq!(frame.text, text);
    assert_eq!(frame.cursor, expected);
    assert_eq!(frame.changed, changed);
}

fn editor(
    text: &str,
    cursor_range: CCursorRange,
) -> Result<ReferenceEditor, Box<dyn std::error::Error>> {
    ReferenceEditor::new(EDIT_ID_SOURCE, text, cursor_range, true)
}

fn moved(
    text: &str,
    cursor_range: CCursorRange,
    key: Key,
    modifiers: Modifiers,
) -> Result<FrameObservation, Box<dyn std::error::Error>> {
    observation(text, cursor_range, Some(key_event(key, modifiers)), true)
}

#[test]
fn arrow_left_and_right_collapse_forward_selection_to_min_or_max()
-> Result<(), Box<dyn std::error::Error>> {
    let selection = selection(SAMPLE_END, SAMPLE_START);
    let left = moved(SAMPLE_TEXT, selection, Key::ArrowLeft, Modifiers::NONE)?;
    assert_frame(&left, SAMPLE_TEXT, cursor(SAMPLE_START), false);

    let right = moved(SAMPLE_TEXT, selection, Key::ArrowRight, Modifiers::NONE)?;
    assert_frame(&right, SAMPLE_TEXT, cursor(SAMPLE_END), false);
    Ok(())
}

#[test]
fn arrow_collapse_uses_min_or_max_for_reverse_selection() -> Result<(), Box<dyn std::error::Error>>
{
    let selection = selection(SAMPLE_START, SAMPLE_END);
    let left = moved(SAMPLE_TEXT, selection, Key::ArrowLeft, Modifiers::NONE)?;
    assert_frame(&left, SAMPLE_TEXT, cursor(SAMPLE_START), false);

    let right = moved(SAMPLE_TEXT, selection, Key::ArrowRight, Modifiers::NONE)?;
    assert_frame(&right, SAMPLE_TEXT, cursor(SAMPLE_END), false);
    Ok(())
}

#[test]
fn shift_arrows_extend_primary_and_keep_secondary() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = editor(SAMPLE_TEXT, cursor(SAMPLE_MIDDLE))?;
    let left = editor.frame(vec![key_event(Key::ArrowLeft, modifiers(true, false))])?;
    assert_frame(
        &left,
        SAMPLE_TEXT,
        selection(SAMPLE_START, SAMPLE_MIDDLE),
        false,
    );

    let right = editor.frame(vec![key_event(Key::ArrowRight, modifiers(true, false))])?;
    assert_frame(
        &right,
        SAMPLE_TEXT,
        selection(SAMPLE_MIDDLE, SAMPLE_MIDDLE),
        false,
    );
    Ok(())
}

#[test]
fn home_and_end_move_to_current_line_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let home = moved(LINE_TEXT, cursor(LINE_END), Key::Home, Modifiers::NONE)?;
    assert_frame(&home, LINE_TEXT, cursor(LINE_START), false);

    let end = moved(LINE_TEXT, cursor(LINE_START), Key::End, Modifiers::NONE)?;
    assert_frame(&end, LINE_TEXT, cursor(LINE_END), false);
    Ok(())
}

#[test]
fn shift_home_and_end_extend_from_the_primary_cursor() -> Result<(), Box<dyn std::error::Error>> {
    let home = moved(
        LINE_TEXT,
        cursor(LINE_END),
        Key::Home,
        modifiers(true, false),
    )?;
    assert_frame(&home, LINE_TEXT, selection(LINE_START, LINE_END), false);

    let end = moved(
        LINE_TEXT,
        cursor(LINE_START),
        Key::End,
        modifiers(true, false),
    )?;
    assert_frame(&end, LINE_TEXT, selection(LINE_END, LINE_START), false);
    Ok(())
}

#[test]
fn command_a_selects_the_full_scalar_range_without_changing_text()
-> Result<(), Box<dyn std::error::Error>> {
    let result = moved(
        SAMPLE_TEXT,
        cursor(SAMPLE_MIDDLE),
        Key::A,
        modifiers(false, true),
    )?;
    assert_frame(
        &result,
        SAMPLE_TEXT,
        selection(SAMPLE_TEXT.chars().count(), EMPTY),
        false,
    );
    Ok(())
}

#[test]
fn released_navigation_key_does_not_move_or_report_change() -> Result<(), Box<dyn std::error::Error>>
{
    let mut editor = editor(SAMPLE_TEXT, cursor(SAMPLE_MIDDLE))?;
    let result = editor.frame(vec![released_key(Key::ArrowLeft)])?;
    assert_frame(&result, SAMPLE_TEXT, cursor(SAMPLE_MIDDLE), false);
    Ok(())
}
