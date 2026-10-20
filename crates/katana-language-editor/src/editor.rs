use crate::{CursorPosition, EditorEvent, TextContent};

/// 設定更新の所有者をKUCとhostから奪わないため、旧mutation APIを公開しない。
///
/// ```compile_fail
/// use katana_language_editor::{CursorPosition, EditorEvent, EditorResult, LanguageEditor,
///     TextContent};
///
/// struct LegacyEditor { content: TextContent }
///
/// impl LanguageEditor for LegacyEditor {
///     fn content(&self) -> &TextContent { &self.content }
///     fn set_content(&mut self, content: TextContent) { self.content = content; }
///     fn cursor(&self) -> CursorPosition { CursorPosition::default() }
///     fn poll_events(&mut self) -> Vec<EditorEvent> { Vec::new() }
///     fn apply_settings(&mut self, _settings: ()) -> EditorResult<()> {
///         Ok(())
///     }
/// }
/// ```
pub trait LanguageEditor {
    fn content(&self) -> &TextContent;
    fn set_content(&mut self, content: TextContent);
    fn cursor(&self) -> CursorPosition;
    fn poll_events(&mut self) -> Vec<EditorEvent>;
}
