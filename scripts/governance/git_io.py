#!/usr/bin/env python3
"""Small external-I/O helpers used by the governance CLI."""

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path
from urllib.parse import urlparse

from dependency import dependency_changed, dependency_entries
from policy import PolicyError, validate_parsed_trailers


def run_git(root: Path, *args: str, input_text: str | None = None) -> str:
    result = subprocess.run(["git", *args], cwd=root, input=input_text, text=True, capture_output=True, check=False)
    if result.returncode:
        detail = result.stderr.strip() or result.stdout.strip()
        raise PolicyError(f"git {' '.join(args)} failed: {detail}")
    return result.stdout


def parse_git_trailers(root: Path, message: str):
    result = subprocess.run(
        ["git", "interpret-trailers", "--parse", "--no-divider"],
        cwd=root, input=message, text=True, capture_output=True, check=False,
    )
    if result.returncode:
        raise PolicyError(f"git interpret-trailers failed: {result.stderr.strip()}")
    return validate_parsed_trailers(result.stdout.splitlines())


def origin_identity(root: Path) -> str:
    url = run_git(root, "remote", "get-url", "origin").strip()
    if url.startswith("git@github.com:"):
        path = url.removeprefix("git@github.com:")
    elif url.startswith(("https://github.com/", "ssh://git@github.com/")):
        parsed = urlparse(url)
        if parsed.hostname != "github.com" or parsed.query or parsed.fragment:
            raise PolicyError("origin is not a GitHub repository")
        path = parsed.path.removeprefix("/")
    else:
        raise PolicyError("origin is not a GitHub repository")
    path = path.removesuffix(".git")
    parts = path.split("/")
    if len(parts) != 2 or not all(parts):
        raise PolicyError("origin is not a GitHub repository")
    return path


def github_api(repo: str, endpoint: str) -> str:
    gh = shutil.which("gh")
    if gh is None:
        raise PolicyError("gh is required for read-only GitHub verification")
    path = f"repos/{repo}" if not endpoint else f"repos/{repo}/{endpoint}"
    result = subprocess.run([gh, "--hostname", "github.com", "api", path], text=True, capture_output=True, check=False)
    if result.returncode:
        raise PolicyError(f"GitHub API request failed: {result.stderr.strip() or result.stdout.strip()}")
    return result.stdout


def default_branch(root: Path, repo: str) -> str:
    raw = github_api(repo, "")
    try:
        branch = json.loads(raw)["default_branch"]
    except (ValueError, KeyError, TypeError) as exc:
        raise PolicyError("GitHub repository JSON has no valid default_branch") from exc
    if not isinstance(branch, str) or not branch:
        raise PolicyError("GitHub repository JSON has no valid default_branch")
    return branch


def blob(root: Path, revision: str, path: str) -> bytes | None:
    result = subprocess.run(["git", "show", f"{revision}:{path}"], cwd=root, capture_output=True)
    if result.returncode == 0:
        return result.stdout
    if path not in tracked_files(root, revision):
        return None
    detail = result.stderr.decode(errors="replace").strip()
    raise PolicyError(f"git show {revision}:{path} failed: {detail}")


def tracked_files(root: Path, revision: str) -> set[str]:
    return set(run_git(root, "ls-tree", "-r", "--name-only", revision).splitlines())


def changed_manifests(root: Path, parent: str, commit: str) -> bool:
    paths = sorted(path for path in tracked_files(root, parent) | tracked_files(root, commit) if Path(path).name == "Cargo.toml")
    before_root = blob(root, parent, "Cargo.toml")
    after_root = blob(root, commit, "Cargo.toml")
    before = {}
    after = {}
    for path in paths:
        before[path] = dependency_entries(blob(root, parent, path), _workspace(before_root))
        after[path] = dependency_entries(blob(root, commit, path), _workspace(after_root))
    return before != after


def _workspace(raw: bytes | None) -> dict[str, object]:
    if raw is None:
        return {}
    import tomllib
    try:
        data = tomllib.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as exc:
        raise PolicyError("malformed root Cargo.toml workspace") from exc
    workspace = data.get("workspace", {})
    return workspace.get("dependencies", {}) if isinstance(workspace, dict) and isinstance(workspace.get("dependencies", {}), dict) else {}
