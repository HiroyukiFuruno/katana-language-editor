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


def write_consumer(root: Path, package: dict[str, object], module: str, function: str) -> Path:
    manifest = Path(package["manifest_path"])
    package_name = json.dumps(package["name"])
    dependency_path = json.dumps(str(manifest.parent))
    (root / "Cargo.toml").write_text(
        "[package]\nname = \"kle-kdv-api-audit\"\nversion = \"0.0.0\"\nedition = \"2024\"\n"
        "\n[dependencies]\nkdv_audit_target = { package = "
        f"{package_name}, path = {dependency_path} }}\n",
        encoding="utf-8",
    )
    source = root / "src"
    source.mkdir()
    (source / "main.rs").write_text(
        f"fn main() {{ let _ = kdv_audit_target::{module}::{function}(); }}\n",
        encoding="utf-8",
    )
    return root / "Cargo.toml"


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
            "--no-deps",
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
    package = active_library_package(json.loads(metadata.stdout), args.source.resolve())
    if package is None:
        return 1

    with tempfile.TemporaryDirectory(prefix="kle-kdv-api-audit-") as directory:
        root = Path(directory)
        consumer_manifest = write_consumer(root, package, args.module, args.function)
        environment = os.environ | {"CARGO_TARGET_DIR": str(root / "target")}
        return subprocess.run(
            ["cargo", "check", "--quiet", "--manifest-path", str(consumer_manifest)],
            env=environment,
            check=False,
        ).returncode


if __name__ == "__main__":
    raise SystemExit(main())
