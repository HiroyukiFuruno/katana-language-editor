use egui::{
    Event, Key, Modifiers,
    text::{CCursor, CCursorRange},
};

#[path = "support/reference_history.rs"]
mod history_cases;
#[path = "support/reference_delete_modifiers.rs"]
mod modifier_cases;
#[path = "support/reference_navigation.rs"]
mod navigation_cases;
#[path = "support/reference_navigation_modifiers.rs"]
mod navigation_modifier_cases;
#[path = "support/reference_navigation_unicode.rs"]
mod navigation_unicode_cases;
#[path = "support/reference_editor.rs"]
mod reference_editor;

use reference_editor::{FrameObservation, ReferenceEditor, cursor};

const EDIT_ID_SOURCE: &str = "fixed-egui-reference-delete";
// KatanA source-closure側の固定0.36.1証跡とは分離し、ここではKLE runtimeのlockを固定する。
const EGUI_VERSION: &str = "0.36.2";
const EGUI_SOURCE: &str = "registry+https://github.com/rust-lang/crates.io-index";
const EGUI_CHECKSUM: &str = "dc938cc27cd911415e1e4d151bc56ff9df063d8db081374e87aeb805ea74b2ba";

fn key_event(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn observation(
    initial_text: &str,
    cursor: CCursorRange,
    event: Option<Event>,
    interactive: bool,
) -> Result<FrameObservation, Box<dyn std::error::Error>> {
    let mut editor = ReferenceEditor::new(EDIT_ID_SOURCE, initial_text, cursor, interactive)?;
    editor.frame(event.into_iter().collect())
}

#[test]
fn fixed_reference_egui_identity() -> Result<(), Box<dyn std::error::Error>> {
    let lock: toml::Value = toml::from_str(include_str!("../../../Cargo.lock"))?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or("Cargo.lock package array missing")?;
    let egui_packages: Vec<&toml::Value> = packages
        .iter()
        .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some("egui"))
        .collect();

    assert_eq!(egui_packages.len(), 1);
    let package = egui_packages[0];
    assert_eq!(
        package.get("version").and_then(toml::Value::as_str),
        Some(EGUI_VERSION)
    );
    assert_eq!(
        package.get("source").and_then(toml::Value::as_str),
        Some(EGUI_SOURCE)
    );
    assert_eq!(
        package.get("checksum").and_then(toml::Value::as_str),
        Some(EGUI_CHECKSUM)
    );
    Ok(())
}

#[test]
fn backspace_deletes_previous_scalar_interior() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation("abcd", cursor(2), Some(key_event(Key::Backspace)), true)?;
    assert_eq!(result.text, "acd");
    assert_eq!(result.cursor, cursor(1));
    assert!(result.changed);
    Ok(())
}

#[test]
fn delete_deletes_next_scalar_interior() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation("abcd", cursor(2), Some(key_event(Key::Delete)), true)?;
    assert_eq!(result.text, "abd");
    assert_eq!(result.cursor, cursor(2));
    assert!(result.changed);
    Ok(())
}

#[test]
fn deletion_at_boundaries_preserves_text_but_reports_changed()
-> Result<(), Box<dyn std::error::Error>> {
    let backspace = observation("abc", cursor(0), Some(key_event(Key::Backspace)), true)?;
    assert_eq!(backspace.text, "abc");
    assert_eq!(backspace.cursor, cursor(0));
    assert!(backspace.changed);

    let delete = observation("abc", cursor(3), Some(key_event(Key::Delete)), true)?;
    assert_eq!(delete.text, "abc");
    assert_eq!(delete.cursor, cursor(3));
    assert!(delete.changed);
    Ok(())
}

#[test]
fn idle_frame_preserves_text_without_change_notification() -> Result<(), Box<dyn std::error::Error>>
{
    let result = observation("abc", cursor(0), None, true)?;
    assert_eq!(result.text, "abc");
    assert_eq!(result.cursor, cursor(0));
    assert!(!result.changed);
    Ok(())
}

#[test]
fn selection_is_deleted_by_backspace_and_delete() -> Result<(), Box<dyn std::error::Error>> {
    let selection = CCursorRange::two(CCursor::new(1), CCursor::new(3));
    let backspace = observation("abcd", selection, Some(key_event(Key::Backspace)), true)?;
    assert_eq!(backspace.text, "ad");
    assert_eq!(backspace.cursor, cursor(1));
    assert!(backspace.changed);

    let delete = observation("abcd", selection, Some(key_event(Key::Delete)), true)?;
    assert_eq!(delete.text, "ad");
    assert_eq!(delete.cursor, cursor(1));
    assert!(delete.changed);
    Ok(())
}

#[test]
fn noninteractive_text_ignores_delete_event() -> Result<(), Box<dyn std::error::Error>> {
    let result = observation("abcd", cursor(2), Some(key_event(Key::Backspace)), false)?;
    assert_eq!(result.text, "abcd");
    assert_eq!(result.cursor, cursor(2));
    assert!(!result.changed);
    Ok(())
}

#[test]
fn nonascii_observation_is_scalar_based() -> Result<(), Box<dyn std::error::Error>> {
    let star_with_variation_selector = "\u{2b50}\u{fe0f}";
    let input = format!("A{star_with_variation_selector}B");
    let result = observation(&input, cursor(3), Some(key_event(Key::Backspace)), true)?;
    assert_eq!(result.text, "A\u{2b50}B");
    assert_eq!(result.cursor, cursor(2));
    assert!(result.changed);
    Ok(())
}
