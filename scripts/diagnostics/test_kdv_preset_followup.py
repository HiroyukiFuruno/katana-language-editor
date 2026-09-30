import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPOSITORY_ROOT / "scripts/release/prepare-kdv-preset-followup.sh"


class KdvPresetFollowupTests(unittest.TestCase):
    def create_kdv_repository(
        self,
        root: Path,
        strings_visibility: str,
        strings_export: str = "pub",
        viewer_is_workspace_member: bool = True,
        strings_is_impl_method: bool = False,
    ) -> Path:
        repository = root / "katana-document-viewer"
        source = repository / "crates/viewer/src"
        editor_types = repository / "crates/kle-types/src"
        source.mkdir(parents=True)
        editor_types.mkdir(parents=True)
        members = (
            '["crates/viewer", "crates/kle-types"]'
            if viewer_is_workspace_member
            else '["crates/kle-types"]'
        )
        (repository / "Cargo.toml").write_text(
            f"[workspace]\nmembers = {members}\nresolver = \"2\"\n", encoding="utf-8"
        )
        (source.parent / "Cargo.toml").write_text(
            '[package]\nname = "viewer"\nversion = "0.1.0"\nedition = "2024"\n'
            '[dependencies]\nkatana-language-editor = { path = "../kle-types" }\n',
            encoding="utf-8",
        )
        (editor_types.parent / "Cargo.toml").write_text(
            '[package]\nname = "katana-language-editor"\nversion = "0.1.0"\nedition = "2024"\n',
            encoding="utf-8",
        )
        (editor_types / "lib.rs").write_text(
            'pub struct Strings;\npub struct Locale;\npub struct EditorSettings;\n',
            encoding="utf-8",
        )
        (source / "lib.rs").write_text(
            f"{strings_export} mod strings;\npub mod locale;\npub mod settings;\n",
            encoding="utf-8",
        )
        self.write_module(source, "strings", strings_visibility, "en", "Strings", strings_is_impl_method)
        self.write_module(source, "locale", "pub", "en_ltr", "Locale")
        self.write_module(source, "settings", "pub", "default_editor", "EditorSettings")
        return repository

    def write_module(
        self,
        source: Path,
        module: str,
        visibility: str,
        function: str,
        return_type: str,
        is_impl_method: bool = False,
    ) -> None:
        contents = (
            f"{visibility} fn {function}() -> katana_language_editor::{return_type} "
            f"{{ katana_language_editor::{return_type} }}\n"
        )
        if is_impl_method:
            contents = (
                "pub struct Presets;\n"
                f"impl Presets {{ {visibility} fn {function}() -> "
                f"katana_language_editor::{return_type} {{ katana_language_editor::{return_type} }} }}\n"
            )
        (source / f"{module}.rs").write_text(
            contents, encoding="utf-8"
        )

    def run_followup(self, repository: Path, output: Path) -> str:
        subprocess.run(
            [str(SCRIPT), "v0.1.0", str(repository), str(output)],
            check=True,
            cwd=REPOSITORY_ROOT,
            text=True,
            capture_output=True,
        )
        return output.read_text(encoding="utf-8")

    def test_accepts_only_unrestricted_public_preset_functions(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            public_output = root / "public.md"
            public_repository = self.create_kdv_repository(root / "public", "pub")
            self.assertIn(
                "status: follow-up not required",
                self.run_followup(public_repository, public_output),
            )

            restricted_output = root / "restricted.md"
            restricted_repository = self.create_kdv_repository(root / "restricted", "pub(crate)")
            restricted_result = self.run_followup(restricted_repository, restricted_output)
            self.assertIn("status: follow-up required", restricted_result)
            self.assertIn("- `strings::en()`", restricted_result)

            unexported_output = root / "unexported.md"
            unexported_repository = self.create_kdv_repository(
                root / "unexported", "pub", "mod"
            )
            unexported_result = self.run_followup(unexported_repository, unexported_output)
            self.assertIn("status: follow-up required", unexported_result)
            self.assertIn("- `strings::en()`", unexported_result)

            inactive_output = root / "inactive.md"
            inactive_repository = self.create_kdv_repository(
                root / "inactive", "pub", viewer_is_workspace_member=False
            )
            inactive_result = self.run_followup(inactive_repository, inactive_output)
            self.assertIn("status: follow-up required", inactive_result)
            self.assertIn("- `strings::en()`", inactive_result)

            method_output = root / "method.md"
            method_repository = self.create_kdv_repository(
                root / "method", "pub", strings_is_impl_method=True
            )
            method_result = self.run_followup(method_repository, method_output)
            self.assertIn("status: follow-up required", method_result)
            self.assertIn("- `strings::en()`", method_result)


if __name__ == "__main__":
    unittest.main()
