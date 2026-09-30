import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPOSITORY_ROOT / "scripts/release/prepare-kdv-preset-followup.sh"


class KdvPresetFollowupTests(unittest.TestCase):
    def create_kdv_repository(
        self, root: Path, strings_visibility: str, strings_export: str = "pub"
    ) -> Path:
        repository = root / "katana-document-viewer"
        source = repository / "crates/viewer/src"
        source.mkdir(parents=True)
        (repository / "Cargo.toml").write_text(
            '[package]\nname = "viewer"\nversion = "0.1.0"\n', encoding="utf-8"
        )
        (source / "lib.rs").write_text(
            f"{strings_export} mod strings;\npub mod locale;\npub mod settings;\n",
            encoding="utf-8",
        )
        self.write_module(source, "strings", strings_visibility, "en")
        self.write_module(source, "locale", "pub", "en_ltr")
        self.write_module(source, "settings", "pub", "default_editor")
        return repository

    def write_module(self, source: Path, module: str, visibility: str, function: str) -> None:
        (source / f"{module}.rs").write_text(
            f"{visibility} fn {function}() {{}}\n", encoding="utf-8"
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


if __name__ == "__main__":
    unittest.main()
