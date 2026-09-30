use egui::{Event, Key, Modifiers, text::CCursorRange};

use super::cursor;
use super::reference_editor::{FrameObservation, ReferenceEditor};

const EDIT_ID_SOURCE: &str = "fixed-egui-reference-navigation-unicode";
const UNICODE_TEXT: &str = "A⭐️日B";
const START: usize = 1;
const AFTER_STAR: usize = 2;
const AFTER_VS16: usize = 3;
const AFTER_JAPANESE: usize = 4;

fn key_event(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    }
}

fn shift_right() -> Event {
    key_event(
        Key::ArrowRight,
        Modifiers {
            shift: true,
            ..Modifiers::NONE
        },
    )
}

fn selection(primary: usize, secondary: usize) -> CCursorRange {
    CCursorRange {
        primary: cursor(primary).primary,
        secondary: cursor(secondary).primary,
        h_pos: None,
    }
}

fn assert_frame(frame: &FrameObservation, expected: CCursorRange) {
    assert_eq!(frame.text, UNICODE_TEXT);
    assert_eq!(frame.cursor, expected);
    assert!(!frame.changed);
}

fn editor() -> Result<ReferenceEditor, Box<dyn std::error::Error>> {
    ReferenceEditor::new(EDIT_ID_SOURCE, UNICODE_TEXT, cursor(START), true)
}

#[test]
fn unicode_navigation_tracks_vs16_selection_collapse_and_japanese_scalar()
-> Result<(), Box<dyn std::error::Error>> {
    let mut editor = editor()?;
    let after_star = editor.frame(vec![shift_right()])?;
    assert_frame(&after_star, selection(AFTER_STAR, START));

    let after_vs16 = editor.frame(vec![shift_right()])?;
    assert_frame(&after_vs16, selection(AFTER_VS16, START));

    let collapse = editor.frame(vec![key_event(Key::ArrowRight, Modifiers::NONE)])?;
    assert_frame(&collapse, cursor(AFTER_VS16));

    let after_japanese = editor.frame(vec![key_event(Key::ArrowRight, Modifiers::NONE)])?;
    assert_frame(&after_japanese, cursor(AFTER_JAPANESE));
    Ok(())
}
