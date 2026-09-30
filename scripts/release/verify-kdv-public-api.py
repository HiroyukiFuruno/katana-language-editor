#!/usr/bin/env python3
import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


def active_library_package(metadata: dict[str, object], source: Path) -> dict[str, object] | None:
    for package in metadata["packages"]:
        for target in package["targets"]:
            if "lib" in target["kind"] and Path(target["src_path"]).resolve() == source:
                return package
    return None


def active_library_packages(metadata: dict[str, object]) -> list[dict[str, object]]:
    workspace_members = metadata.get("workspace_members")
    if not isinstance(workspace_members, list):
        return []
    workspace_ids = {member for member in workspace_members if isinstance(member, str)}
    packages: list[dict[str, object]] = []
    for package in metadata["packages"]:
        if package.get("id") in workspace_ids and any(
            "lib" in target["kind"] for target in package["targets"]
        ):
            packages.append(package)
    return packages


def editor_types_package(
    metadata: dict[str, object], audited_package: dict[str, object]
) -> dict[str, object] | None:
    audited_id = audited_package.get("id")
    resolve = metadata.get("resolve")
    if not isinstance(audited_id, str) or not isinstance(resolve, dict):
        return None
    nodes = resolve.get("nodes")
    if not isinstance(nodes, list):
        return None
    node = next((node for node in nodes if node.get("id") == audited_id), None)
    if not isinstance(node, dict):
        return None
    dependencies = node.get("deps")
    if not isinstance(dependencies, list):
        return None
    package_ids = {
        dependency.get("pkg")
        for dependency in dependencies
        if isinstance(dependency.get("pkg"), str)
    }
    return next(
        (
            package
            for package in metadata["packages"]
            if package.get("id") in package_ids
            and package.get("name") == "katana-language-editor"
        ),
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
    parser.add_argument("--source", type=Path)
    parser.add_argument("--module", required=True)
    parser.add_argument("--function", required=True)
    args = parser.parse_args()

    metadata = subprocess.run(
        [
            "cargo",
            "metadata",
            "--locked",
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
    if args.source is None:
        packages = active_library_packages(resolved_metadata)
    else:
        package = active_library_package(resolved_metadata, args.source.resolve())
        packages = [] if package is None else [package]

    failures: list[subprocess.CompletedProcess[str]] = []
    for package in packages:
        editor_types = editor_types_package(resolved_metadata, package)
        if editor_types is None:
            continue
        with tempfile.TemporaryDirectory(prefix="kle-kdv-api-audit-") as directory:
            root = Path(directory)
            try:
                consumer_manifest = write_consumer(
                    root, package, editor_types, args.module, args.function
                )
            except ValueError:
                continue
            result = subprocess.run(
                ["cargo", "check", "--quiet", "--manifest-path", str(consumer_manifest)],
                capture_output=True,
                text=True,
                check=False,
            )
            if result.returncode == 0:
                return 0
            failures.append(result)

    for result in failures:
        print(result.stdout, end="")
        print(result.stderr, end="", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
