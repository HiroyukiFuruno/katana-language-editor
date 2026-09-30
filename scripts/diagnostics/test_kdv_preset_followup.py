import importlib.util
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPOSITORY_ROOT / "scripts/release/prepare-kdv-preset-followup.sh"
API_CHECKER = REPOSITORY_ROOT / "scripts/release/verify-kdv-public-api.py"


def shell_bash() -> str:
    if os.name == "nt":
        git_bash = Path(os.environ.get("ProgramFiles", r"C:\\Program Files")) / "Git/bin/bash.exe"
        if git_bash.is_file():
            return str(git_bash)
    resolved = shutil.which("bash")
    if resolved is None:
        raise RuntimeError("bash is unavailable")
    return resolved


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
        inline_preset_modules: bool = False,
        editor_types_dependency_name: str = "katana-language-editor",
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
        editor_types_dependency = (
            'katana-language-editor = { path = "../kle-types" }'
            if editor_types_dependency_name == "katana-language-editor"
            else f'{editor_types_dependency_name} = {{ package = "katana-language-editor", path = "../kle-types" }}'
        )
        (source.parent / "Cargo.toml").write_text(
            '[package]\nname = "viewer"\nversion = "0.1.0"\nedition = "2024"\n'
            f'[dependencies]\n{editor_types_dependency}\n',
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
        if inline_preset_modules:
            (source / "lib.rs").write_text(
                (
                    "extern crate kle as katana_language_editor;\n"
                    if editor_types_dependency_name == "kle"
                    else ""
                )
                + "pub mod strings { pub fn en() -> katana_language_editor::Strings { "
                "katana_language_editor::Strings } }\n"
                "pub mod locale { pub fn en_ltr() -> katana_language_editor::Locale { "
                "katana_language_editor::Locale } }\n"
                "pub struct ViewerSettingsState;\n"
                "pub mod settings { pub fn default_editor() -> crate::ViewerSettingsState { "
                "crate::ViewerSettingsState } }\n",
                encoding="utf-8",
            )
            self.generate_lockfile(repository)
            return repository
        (source / "lib.rs").write_text(
            (
                "extern crate kle as katana_language_editor;\n"
                if editor_types_dependency_name == "kle"
                else ""
            )
            + f"{strings_export} mod strings;\npub mod locale;\npub mod settings;\n",
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
        self.generate_lockfile(repository)
        return repository

    def generate_lockfile(self, repository: Path) -> None:
        subprocess.run(
            ["cargo", "generate-lockfile", "--manifest-path", str(repository / "Cargo.toml")],
            check=True,
            capture_output=True,
            text=True,
        )

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
        command = [shell_bash(), str(SCRIPT), "v0.1.0", str(repository), str(output)]
        lockfile = repository / "Cargo.lock"
        lockfile_before = lockfile.read_bytes()
        if without_rg:
            with tempfile.TemporaryDirectory() as guard_directory:
                guard_log = Path(guard_directory) / "rg-invocations.log"
                guard = Path(guard_directory) / "rg"
                guard.write_text(
                    "#!/usr/bin/env sh\nprintf 'rg invoked\\n' >> \"${KLE_RG_GUARD_LOG}\"\nexit 97\n",
                    encoding="utf-8",
                )
                guard.chmod(0o755)
                environment["KLE_RG_GUARD_LOG"] = str(guard_log)
                environment["PATH"] = os.pathsep.join(
                    [guard_directory, environment.get("PATH", "")]
                )
                result = subprocess.run(
                    command, cwd=REPOSITORY_ROOT, text=True, capture_output=True, env=environment
                )
                self.assertFalse(guard_log.exists(), "the follow-up script must not invoke rg")
        else:
            result = subprocess.run(
                command, cwd=REPOSITORY_ROOT, text=True, capture_output=True, env=environment
            )
        if result.returncode != 0:
            self.fail(f"follow-up script failed:\n{result.stdout}\n{result.stderr}")
        self.assertEqual(lockfile.read_bytes(), lockfile_before)
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

            inline_output = root / "inline.md"
            inline_repository = self.create_kdv_repository(
                root / "inline", "pub", inline_preset_modules=True
            )
            self.assertIn(
                "status: follow-up not required",
                self.run_followup(inline_repository, inline_output),
            )

            renamed_output = root / "renamed.md"
            renamed_repository = self.create_kdv_repository(
                root / "renamed", "pub", editor_types_dependency_name="kle"
            )
            self.assertIn(
                "status: follow-up not required",
                self.run_followup(renamed_repository, renamed_output),
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
