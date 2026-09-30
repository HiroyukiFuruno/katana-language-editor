use egui::{Event, Key, Modifiers, text::CCursorRange};

use super::{cursor, observation};

fn key_event(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    }
}

fn modifiers_with(alt: bool, ctrl: bool, mac_cmd: bool) -> Modifiers {
    Modifiers {
        alt,
        ctrl,
        mac_cmd,
        command: mac_cmd,
        ..Modifiers::NONE
    }
}

#[test]
fn alt_backspace_deletes_previous_word() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation(
        "one two three",
        cursor(13),
        Some(key_event(
            Key::Backspace,
            modifiers_with(true, false, false),
        )),
        true,
    )?;
    assert_eq!(result.text, "one two ");
    assert_eq!(result.cursor, cursor(8));
    assert!(result.changed);
    Ok(())
}

#[test]
fn ctrl_delete_deletes_next_word() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation(
        "one two three",
        cursor(4),
        Some(key_event(Key::Delete, modifiers_with(false, true, false))),
        true,
    )?;
    assert_eq!(result.text, "one  three");
    assert_eq!(result.cursor, cursor(4));
    assert!(result.changed);
    Ok(())
}

#[test]
fn ctrl_backspace_deletes_previous_word() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation(
        "one two three",
        cursor(7),
        Some(key_event(
            Key::Backspace,
            modifiers_with(false, true, false),
        )),
        true,
    )?;
    assert_eq!(result.text, "one  three");
    assert_eq!(result.cursor, cursor(4));
    assert!(result.changed);
    Ok(())
}

#[test]
fn alt_delete_deletes_next_word() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation(
        "one two three",
        cursor(4),
        Some(key_event(Key::Delete, modifiers_with(true, false, false))),
        true,
    )?;
    assert_eq!(result.text, "one  three");
    assert_eq!(result.cursor, cursor(4));
    assert!(result.changed);
    Ok(())
}

#[test]
fn mac_cmd_backspace_deletes_to_paragraph_start() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation(
        "one two\nthree",
        cursor(10),
        Some(key_event(
            Key::Backspace,
            modifiers_with(false, false, true),
        )),
        true,
    )?;
    assert_eq!(result.text, "one two\nree");
    assert_eq!(result.cursor, cursor(8));
    assert!(result.changed);
    Ok(())
}

#[test]
fn mac_cmd_delete_deletes_to_paragraph_end() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation(
        "one two\nthree",
        cursor(5),
        Some(key_event(Key::Delete, modifiers_with(false, false, true))),
        true,
    )?;
    assert_eq!(result.text, "one t\nthree");
    assert_eq!(result.cursor, cursor(5));
    assert!(result.changed);
    Ok(())
}

#[test]
fn ctrl_h_deletes_previous_character() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation(
        "one two\nthree",
        cursor(5),
        Some(key_event(Key::H, modifiers_with(false, true, false))),
        true,
    )?;
    assert_eq!(result.text, "one wo\nthree");
    assert_eq!(result.cursor, cursor(4));
    assert!(result.changed);
    Ok(())
}

#[test]
fn ctrl_k_deletes_to_paragraph_end() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation(
        "one two\nthree",
        cursor(5),
        Some(key_event(Key::K, modifiers_with(false, true, false))),
        true,
    )?;
    assert_eq!(result.text, "one t\nthree");
    assert_eq!(result.cursor, cursor(5));
    assert!(result.changed);
    Ok(())
}

#[test]
fn ctrl_u_deletes_to_paragraph_start() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation(
        "one two\nthree",
        cursor(5),
        Some(key_event(Key::U, modifiers_with(false, true, false))),
        true,
    )?;
    assert_eq!(result.text, "wo\nthree");
    assert_eq!(result.cursor, cursor(0));
    assert!(result.changed);
    Ok(())
}

#[test]
fn ctrl_w_deletes_previous_word_without_selection() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation(
        "one two three",
        cursor(7),
        Some(key_event(Key::W, modifiers_with(false, true, false))),
        true,
    )?;
    assert_eq!(result.text, "one  three");
    assert_eq!(result.cursor, cursor(4));
    assert!(result.changed);
    Ok(())
}

#[test]
fn ctrl_w_deletes_selection() -> Result<(), Box<dyn std::error::Error>> {
    let selection = CCursorRange::two(cursor(4).primary, cursor(7).primary);
    let result = observation(
        "one two three",
        selection,
        Some(key_event(Key::W, modifiers_with(false, true, false))),
        true,
    )?;
    assert_eq!(result.text, "one  three");
    assert_eq!(result.cursor, cursor(4));
    assert!(result.changed);
    Ok(())
}

#[test]
fn modifier_absent_does_not_mutate_control_keys() -> Result<(), Box<dyn std::error::Error>> {
    for key in [Key::H, Key::K, Key::U, Key::W] {
        let result = observation(
            "one two\nthree",
            cursor(5),
            Some(key_event(key, Modifiers::NONE)),
            true,
        )?;
        assert_eq!(result.text, "one two\nthree");
        assert_eq!(result.cursor, cursor(5));
        assert!(!result.changed);
    }
    Ok(())
}
