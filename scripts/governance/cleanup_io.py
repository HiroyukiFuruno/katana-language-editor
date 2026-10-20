#!/usr/bin/env python3
"""External Git/GitHub operations for release cleanup."""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
from pathlib import Path
from urllib.parse import urlparse

from cleanup_policy import CleanupPolicyError, parse_release_payload, registry_version


class CleanupIOError(CleanupPolicyError):
    pass


def command(root: Path, argv: list[str], *, env: dict[str, str] | None = None, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(argv, cwd=root, text=True, capture_output=True, env=env, check=False)
    if check and result.returncode:
        detail = result.stderr.strip() or result.stdout.strip()
        raise CleanupIOError(f"{' '.join(argv)} failed: {detail}")
    return result


def git(root: Path, *args: str, check: bool = True, env: dict[str, str] | None = None) -> str:
    return command(root, ["git", *args], check=check, env=env).stdout


def origin_repo(root: Path) -> str:
    url = git(root, "remote", "get-url", "origin").strip()
    if url.startswith("git@github.com:"):
        path = url.removeprefix("git@github.com:")
    else:
        parsed = urlparse(url)
        if parsed.scheme not in {"https", "ssh"} or parsed.hostname != "github.com" or parsed.query or parsed.fragment:
            raise CleanupIOError("origin must be a GitHub repository on github.com")
        path = parsed.path.removeprefix("/")
    path = path.removesuffix(".git")
    if not re.fullmatch(r"[^/]+/[^/]+", path):
        raise CleanupIOError("origin repository identity is invalid")
    return path


def gh_json(repo: str, endpoint: str) -> tuple[int, str]:
    gh = shutil.which("gh")
    if gh is None:
        raise CleanupIOError("gh is required for public release verification")
    result = subprocess.run([gh, "--hostname", "github.com", "api", f"repos/{repo}/{endpoint}"], text=True, capture_output=True, check=False)
    if result.returncode == 0:
        return 200, result.stdout
    if re.search(r"(?:HTTP )?404|Not Found", result.stderr + result.stdout, re.I):
        return 404, result.stderr + result.stdout
    raise CleanupIOError(f"GitHub API failure (not 404): {result.stderr.strip() or result.stdout.strip()}")


def release_json(repo: str, tag: str, *, latest: bool = False) -> tuple[int, str]:
    return gh_json(repo, "releases/latest" if latest else f"releases/tags/{tag}")


def default_branch(repo: str, root: Path) -> str:
    status, raw = gh_json(repo, "")
    if status != 200:
        raise CleanupIOError("GitHub repository metadata is unavailable")
    try:
        value = json.loads(raw)
        branch = value["default_branch"]
    except (ValueError, KeyError, TypeError) as exc:
        raise CleanupIOError("GitHub repository has no valid default_branch") from exc
    if not isinstance(branch, str) or not branch:
        raise CleanupIOError("GitHub repository has no valid default_branch")
    return branch


def verify_public_release(repo: str, tag: str) -> dict[str, str]:
    status, raw = release_json(repo, tag)
    if status == 404:
        raise CleanupIOError("public GitHub release was not found (404)")
    return parse_release_payload(raw, repo, tag)


def verify_registry_crate(crate: str, version: str) -> int:
    result = subprocess.run(["cargo", "info", "--registry", "crates-io", f"{crate}@{version}"], text=True, capture_output=True, check=False)
    if result.returncode:
        raise CleanupIOError(f"cargo info failed for {crate}@{version}: exit={result.returncode}")
    return result.returncode


def verify_published_crates(tag: str) -> None:
    version = registry_version(tag)
    for crate in ("katana-language-editor", "katana-language-editor-egui"):
        verify_registry_crate(crate, version)


def verify_latest_release(repo: str) -> tuple[str | None, str | None]:
    status, raw = release_json(repo, "", latest=True)
    if status == 404:
        return None, None
    try:
        value = json.loads(raw)
        tag = value["tag_name"]
    except (ValueError, KeyError, TypeError) as exc:
        raise CleanupIOError("latest GitHub release is malformed") from exc
    if not isinstance(tag, str):
        raise CleanupIOError("latest GitHub release has no tag")
    return tag, raw


def ancestor(root: Path, child: str, parent: str) -> bool:
    result = command(root, ["git", "merge-base", "--is-ancestor", child, parent], check=False)
    if result.returncode == 0:
        return True
    if result.returncode == 1:
        return False
    raise CleanupIOError("git merge-base failed; graph facts are not trustworthy")


def current_branch(root: Path) -> str | None:
    result = command(root, ["git", "symbolic-ref", "--quiet", "--short", "HEAD"], check=False)
    if result.returncode == 1:
        return None
    if result.returncode:
        raise CleanupIOError("git symbolic-ref failed while identifying current branch")
    return result.stdout.strip()


def local_branches(root: Path) -> list[tuple[str, str]]:
    raw = git(root, "for-each-ref", "--format=%(refname:short)%00%(objectname)%00", "refs/heads/")
    fields = [field.strip() for field in raw.split("\0") if field.strip()]
    if len(fields) % 2:
        raise CleanupIOError("malformed local branch records")
    return [(fields[i], fields[i + 1]) for i in range(0, len(fields), 2)]


def remote_branches(root: Path) -> list[tuple[str, str]]:
    raw = git(root, "for-each-ref", "--format=%(refname:short)%00%(objectname)%00", "refs/remotes/origin/release/")
    fields = [field.strip() for field in raw.split("\0") if field.strip()]
    if len(fields) % 2:
        raise CleanupIOError("malformed remote branch records")
    return [(fields[i].removeprefix("origin/"), fields[i + 1]) for i in range(0, len(fields), 2)]


def bound_branches(root: Path) -> set[str]:
    raw = git(root, "worktree", "list", "--porcelain", "-z")
    return {field.removeprefix("branch refs/heads/") for field in raw.split("\0") if field.startswith("branch refs/heads/")}


def worktree_dirty(root: Path) -> bool:
    return bool(git(root, "status", "--porcelain", "--untracked-files=all").strip())


def stash_present(root: Path) -> bool:
    return bool(git(root, "stash", "list").strip())


def local_tag_commit(root: Path, tag: str) -> str:
    return git(root, "rev-parse", f"refs/tags/{tag}^{{commit}}").strip()


def remote_tag_commit(root: Path, tag: str) -> str:
    raw = git(root, "ls-remote", "origin", f"refs/tags/{tag}", f"refs/tags/{tag}^{{}}")
    commits = [line.split()[0] for line in raw.splitlines() if line.split()]
    if not commits:
        raise CleanupIOError("origin tag is missing")
    return commits[-1]


def remote_branch_commit(root: Path, branch: str) -> str:
    return git(root, "rev-parse", f"refs/remotes/origin/{branch}").strip()


def live_branch_commit(root: Path, branch: str) -> str:
    raw = git(root, "ls-remote", "origin", f"refs/heads/{branch}")
    fields = raw.split()
    if not fields:
        raise CleanupIOError("origin default branch is missing")
    return fields[0]


def fetch_graph(root: Path, tag: str) -> None:
    git(root, "fetch", "origin")
    git(root, "fetch", "origin", f"refs/tags/{tag}")
