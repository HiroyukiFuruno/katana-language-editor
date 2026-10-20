use egui::{
    Event, Key, Modifiers,
    text::{CCursor, CCursorRange},
};

use super::{FrameObservation, ReferenceEditor, cursor};

const EDIT_ID_SOURCE: &str = "fixed-egui-reference-delete";

fn assert_frame(frame: &FrameObservation, text: &str, index: usize, changed: bool) {
    assert_eq!(frame.text, text);
    assert_eq!(frame.cursor, cursor(index));
    assert_eq!(frame.changed, changed);
}

fn command_key(key: Key, shift: bool) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: Modifiers {
            mac_cmd: true,
            command: true,
            shift,
            ..Modifiers::NONE
        },
    }
}

fn editor(
    text: &str,
    cursor_range: CCursorRange,
) -> Result<ReferenceEditor, Box<dyn std::error::Error>> {
    ReferenceEditor::new(EDIT_ID_SOURCE, text, cursor_range, true)
}

#[test]
fn text_input_undo_and_redo_shortcuts_restore_text_and_cursor()
-> Result<(), Box<dyn std::error::Error>> {
    let mut editor = editor("base", cursor(4))?;
    assert_frame(
        &editor.frame(vec![Event::Text("!".into())])?,
        "base!",
        5,
        true,
    );
    assert_frame(&editor.stable_frame()?, "base!", 5, false);

    let undo = editor.frame(vec![command_key(Key::Z, false)])?;
    assert_eq!(undo.text, "base");
    assert_eq!(undo.cursor, cursor(4));
    assert!(undo.changed);

    let redo_shift_z = editor.frame(vec![command_key(Key::Z, true)])?;
    assert_eq!(redo_shift_z.text, "base!");
    assert_eq!(redo_shift_z.cursor, cursor(5));
    assert!(redo_shift_z.changed);

    assert_frame(
        &editor.frame(vec![command_key(Key::Z, false)])?,
        "base",
        4,
        true,
    );
    let redo_y = editor.frame(vec![command_key(Key::Y, false)])?;
    assert_eq!(redo_y.text, "base!");
    assert_eq!(redo_y.cursor, cursor(5));
    assert!(redo_y.changed);
    Ok(())
}

#[test]
fn undo_restores_selection_replaced_by_text_input() -> Result<(), Box<dyn std::error::Error>> {
    let selection = CCursorRange::two(
        CCursor {
            index: 1.into(),
            prefer_next_row: true,
        },
        CCursor {
            index: 3.into(),
            prefer_next_row: false,
        },
    );
    let mut editor = editor("abcd", selection)?;
    let initial_selection = editor.frame(Vec::new())?.cursor;
    assert_frame(
        &editor.frame(vec![Event::Text("X".into())])?,
        "aXd",
        2,
        true,
    );
    assert_frame(&editor.stable_frame()?, "aXd", 2, false);

    let undo = editor.frame(vec![command_key(Key::Z, false)])?;
    assert_eq!(undo.text, "abcd");
    assert_eq!(undo.cursor, initial_selection);
    assert_eq!(undo.cursor.primary.index, initial_selection.primary.index);
    assert_eq!(
        undo.cursor.primary.prefer_next_row,
        initial_selection.primary.prefer_next_row
    );
    assert_eq!(
        undo.cursor.secondary.index,
        initial_selection.secondary.index
    );
    assert_eq!(
        undo.cursor.secondary.prefer_next_row,
        initial_selection.secondary.prefer_next_row
    );
    assert!(undo.changed);
    Ok(())
}

#[test]
fn undo_and_redo_without_history_are_noops() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = editor("abc", cursor(1))?;
    for event in [command_key(Key::Z, false), command_key(Key::Y, false)] {
        let result = editor.frame(vec![event])?;
        assert_eq!(result.text, "abc");
        assert_eq!(result.cursor, cursor(1));
        assert!(!result.changed);
    }
    Ok(())
}

#[test]
fn edit_after_undo_clears_redo_history() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = editor("a", cursor(1))?;
    assert_frame(&editor.frame(vec![Event::Text("b".into())])?, "ab", 2, true);
    assert_frame(&editor.stable_frame()?, "ab", 2, false);
    assert_frame(
        &editor.frame(vec![command_key(Key::Z, false)])?,
        "a",
        1,
        true,
    );
    assert_frame(&editor.frame(vec![Event::Text("c".into())])?, "ac", 2, true);
    assert_frame(&editor.stable_frame()?, "ac", 2, false);

    let redo = editor.frame(vec![command_key(Key::Y, false)])?;
    assert_eq!(redo.text, "ac");
    assert_eq!(redo.cursor, cursor(2));
    assert!(!redo.changed);
    Ok(())
}
