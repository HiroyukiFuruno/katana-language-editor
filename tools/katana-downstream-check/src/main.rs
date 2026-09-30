mod probe;

use katana_language_editor::EditorResult;
use probe::PublicEditorDiagnostic;

fn main() -> EditorResult<()> {
    let diagnostic = PublicEditorDiagnostic::construct()?;
    println!("{}", diagnostic.summary());
    Ok(())
}
