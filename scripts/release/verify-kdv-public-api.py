#!/usr/bin/env python3
import argparse
import json
import os
import subprocess
import tempfile
from pathlib import Path


def active_library_package(metadata: dict[str, object], source: Path) -> dict[str, object] | None:
    for package in metadata["packages"]:
        for target in package["targets"]:
            if "lib" in target["kind"] and Path(target["src_path"]).resolve() == source:
                return package
    return None


def editor_types_package(metadata: dict[str, object]) -> dict[str, object] | None:
    return next(
        (package for package in metadata["packages"] if package["name"] == "katana-language-editor"),
        None,
    )


def write_consumer(
    root: Path,
    package: dict[str, object],
    editor_types: dict[str, object],
    module: str,
    function: str,
) -> Path:
    manifest = Path(package["manifest_path"])
    package_name = json.dumps(package["name"])
    dependency_path = json.dumps(str(manifest.parent))
    editor_types_dependency = editor_types_dependency_spec(editor_types)
    (root / "Cargo.toml").write_text(
        "[package]\nname = \"kle-kdv-api-audit\"\nversion = \"0.0.0\"\nedition = \"2024\"\n"
        "\n[dependencies]\nkdv_audit_target = { package = "
        f"{package_name}, path = {dependency_path} }}\n"
        "kle_preset_types = { package = \"katana-language-editor\", "
        f"{editor_types_dependency} }}\n",
        encoding="utf-8",
    )
    source = root / "src"
    source.mkdir()
    expected_type = {
        "strings": "kle_preset_types::Strings",
        "locale": "kle_preset_types::Locale",
        "settings": "kdv_audit_target::ViewerSettingsState",
    }[module]
    (source / "main.rs").write_text(
        f"fn main() {{ let _: {expected_type} = "
        f"kdv_audit_target::{module}::{function}(); }}\n",
        encoding="utf-8",
    )
    return root / "Cargo.toml"


def editor_types_dependency_spec(editor_types: dict[str, object]) -> str:
    source = editor_types.get("source")
    if source is None:
        manifest = Path(editor_types["manifest_path"])
        return f"path = {json.dumps(str(manifest.parent))}"
    if source == "registry+https://github.com/rust-lang/crates.io-index":
        return f'version = {json.dumps("=" + editor_types["version"])}'
    raise ValueError("KLE package source is neither a local path nor a registry package")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--module", required=True)
    parser.add_argument("--function", required=True)
    args = parser.parse_args()

    metadata = subprocess.run(
        [
            "cargo",
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            str(args.manifest),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if metadata.returncode != 0:
        return metadata.returncode
    resolved_metadata = json.loads(metadata.stdout)
    package = active_library_package(resolved_metadata, args.source.resolve())
    editor_types = editor_types_package(resolved_metadata)
    if package is None or editor_types is None:
        return 1

    with tempfile.TemporaryDirectory(prefix="kle-kdv-api-audit-") as directory:
        root = Path(directory)
        try:
            consumer_manifest = write_consumer(
                root, package, editor_types, args.module, args.function
            )
        except ValueError:
            return 1
        environment = os.environ | {"CARGO_TARGET_DIR": str(root / "target")}
        return subprocess.run(
            ["cargo", "check", "--quiet", "--manifest-path", str(consumer_manifest)],
            env=environment,
            check=False,
        ).returncode


if __name__ == "__main__":
    raise SystemExit(main())
