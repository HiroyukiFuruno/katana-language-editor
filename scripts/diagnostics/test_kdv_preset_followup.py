import importlib.util
import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPOSITORY_ROOT / "scripts/release/prepare-kdv-preset-followup.sh"
API_CHECKER = REPOSITORY_ROOT / "scripts/release/verify-kdv-public-api.py"


def load_api_checker():
    specification = importlib.util.spec_from_file_location("verify_kdv_public_api", API_CHECKER)
    if specification is None or specification.loader is None:
        raise RuntimeError("unable to load KDV public API checker")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


class KdvPresetFollowupTests(unittest.TestCase):
    def create_kdv_repository(
        self,
        root: Path,
        strings_visibility: str,
        strings_export: str = "pub",
        viewer_is_workspace_member: bool = True,
        strings_is_impl_method: bool = False,
        settings_returns_viewer_state: bool = True,
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
            'pub struct Strings;\npub struct Locale;\n',
            encoding="utf-8",
        )
        (source / "lib.rs").write_text(
            f"{strings_export} mod strings;\npub mod locale;\npub mod settings;\n",
            encoding="utf-8",
        )
        self.write_module(source, "strings", strings_visibility, "en", "Strings", strings_is_impl_method)
        self.write_module(source, "locale", "pub", "en_ltr", "Locale")
        if settings_returns_viewer_state:
            self.write_module(
                source,
                "settings",
                "pub",
                "default_editor",
                "ViewerSettingsState",
                return_from_kdv=True,
            )
        else:
            self.write_module(source, "settings", "pub", "default_editor", "Strings")
        return repository

    def write_module(
        self,
        source: Path,
        module: str,
        visibility: str,
        function: str,
        return_type: str,
        is_impl_method: bool = False,
        return_from_kdv: bool = False,
    ) -> None:
        return_path = (
            f"crate::{return_type}" if return_from_kdv else f"katana_language_editor::{return_type}"
        )
        contents = (
            f"{visibility} fn {function}() -> {return_path} "
            f"{{ {return_path} }}\n"
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
        if return_from_kdv:
            (source / "lib.rs").write_text(
                (source / "lib.rs").read_text(encoding="utf-8")
                + f"pub struct {return_type};\n",
                encoding="utf-8",
            )

    def run_followup(
        self, repository: Path, output: Path, *, without_rg: bool = False
    ) -> str:
        environment = os.environ.copy()
        if without_rg:
            environment["PATH"] = os.pathsep.join(
                directory
                for directory in environment["PATH"].split(os.pathsep)
                if not (Path(directory) / "rg").exists()
            )
        result = subprocess.run(
            ["bash", str(SCRIPT), "v0.1.0", str(repository), str(output)],
            cwd=REPOSITORY_ROOT,
            text=True,
            capture_output=True,
            env=environment,
        )
        if result.returncode != 0:
            self.fail(f"follow-up script failed:\n{result.stdout}\n{result.stderr}")
        artifact = output.read_text(encoding="utf-8")
        return f"{artifact}\n## script stderr\n{result.stderr}"

    def test_accepts_only_unrestricted_public_preset_functions(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            public_output = root / "public.md"
            public_repository = self.create_kdv_repository(root / "public", "pub")
            self.assertIn(
                "status: follow-up not required",
                self.run_followup(public_repository, public_output),
            )

            without_rg_output = root / "without-rg.md"
            self.assertIn(
                "status: follow-up not required",
                self.run_followup(public_repository, without_rg_output, without_rg=True),
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

            wrong_settings_output = root / "wrong-settings.md"
            wrong_settings_repository = self.create_kdv_repository(
                root / "wrong-settings", "pub", settings_returns_viewer_state=False
            )
            wrong_settings_result = self.run_followup(
                wrong_settings_repository, wrong_settings_output
            )
            self.assertIn("status: follow-up required", wrong_settings_result)
            self.assertIn("- `settings::default_editor()`", wrong_settings_result)

    def test_registry_kle_dependency_uses_the_resolved_package_identity(self) -> None:
        api_checker = load_api_checker()
        dependency = api_checker.editor_types_dependency_spec(
            {
                "manifest_path": "/registry/cache/katana-language-editor-0.1.0/Cargo.toml",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "version": "0.1.0",
            }
        )
        self.assertEqual(dependency, 'version = "=0.1.0"')

        with self.assertRaises(ValueError):
            api_checker.editor_types_dependency_spec(
                {
                    "manifest_path": "/registry/cache/katana-language-editor-0.1.0/Cargo.toml",
                    "source": "registry+https://example.invalid/index",
                    "version": "0.1.0",
                }
            )


if __name__ == "__main__":
    unittest.main()
