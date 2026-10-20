#!/usr/bin/env python3
"""Issue #9 commit/push governance guard CLI."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

from dependency import dependency_changed
from git_io import blob, changed_manifests, default_branch, github_api, origin_identity, parse_git_trailers, run_git
from policy import OID_RE, PolicyError, Trailers, require_open_issue, validate_ref, validate_ref_name


def die(message: str) -> int:
    print(f"governance: {message}", file=sys.stderr)
    return 1


def verify_issue(root: Path, number: int) -> None:
    if number < 1:
        raise PolicyError("issue number must be positive")
    repo = origin_identity(root)
    require_open_issue(github_api(repo, f"issues/{number}"), expected_number=number, expected_repo=repo)


def validate_message(path: Path) -> None:
    parse_git_trailers(Path.cwd(), path.read_text(encoding="utf-8"))


def commit_parents(root: Path, commit: str) -> list[str]:
    return run_git(root, "rev-list", "--parents", "-n", "1", commit).split()[1:]


def commit_list(root: Path, old_oid: str, new_oid: str) -> list[str]:
    if re.fullmatch(r"0{40}|0{64}", old_oid):
        return run_git(root, "rev-list", new_oid, "--not", "--remotes=origin").splitlines()
    return run_git(root, "rev-list", f"{old_oid}..{new_oid}").splitlines()


def require_clean_worktree(root: Path) -> None:
    if run_git(root, "status", "--porcelain", "--untracked-files=all").strip():
        raise PolicyError("push validation requires a clean working tree")
    if run_git(root, "stash", "list").strip():
        raise PolicyError("push validation requires an empty stash")


def current_tip_matches(root: Path, ref: str, new_oid: str) -> None:
    head = run_git(root, "rev-parse", "HEAD").strip()
    if ref.startswith("refs/tags/"):
        peeled = run_git(root, "rev-parse", f"{new_oid}^{{commit}}").strip()
        if peeled != head:
            raise PolicyError(f"{ref} does not point at the immutable current HEAD")
    elif new_oid != head:
        raise PolicyError(f"{ref} is not the immutable current HEAD")


def verify_dependency_commit(root: Path, commit: str, trailers: Trailers) -> None:
    parents = commit_parents(root, commit)
    changed = changed_manifests(root, parents[0], commit) if parents else any(
        dependency_changed(None, blob(root, commit, path), after_workspace=blob(root, commit, "Cargo.toml"))
        for path in run_git(root, "ls-tree", "-r", "--name-only", commit).splitlines()
        if Path(path).name == "Cargo.toml"
    )
    repo = origin_identity(root)
    if trailers.downstream_issue is not None:
        require_open_issue(github_api(repo, f"issues/{trailers.downstream_issue}"), expected_number=trailers.downstream_issue, expected_repo=repo, metadata=True)
    if not changed:
        return
    if trailers.downstream_issue is None:
        raise PolicyError(f"commit {commit} changes a katana-* dependency without Downstream-Issue")
    lockfile = "Cargo.lock" in run_git(root, "ls-tree", "-r", "--name-only", "HEAD").splitlines()
    if not lockfile:
        raise PolicyError("katana-* dependency adoption requires a tracked Cargo.lock")
    result = subprocess.run(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=root, text=True, capture_output=True)
    if result.returncode:
        raise PolicyError(f"cargo metadata --locked --no-deps failed: {result.stderr.strip()}")


def parse_push_lines(raw: str) -> list[tuple[str, str, str, str]]:
    lines = raw.splitlines()
    if not lines:
        raise PolicyError("pre-push input must not be empty")
    parsed = []
    for line in lines:
        fields = line.split()
        if len(fields) != 4:
            raise PolicyError("pre-push input must contain four fields per ref")
        local_ref, local_oid, remote_ref, remote_oid = fields
        if not OID_RE.fullmatch(local_oid) or not OID_RE.fullmatch(remote_oid):
            raise PolicyError("pre-push input contains an invalid object id")
        if local_ref == "(delete)" and not re.fullmatch(r"0{40}|0{64}", local_oid):
            raise PolicyError("deletion marker requires a zero local object id")
        parsed.append((local_ref, local_oid, remote_ref, remote_oid))
    return parsed


def validate_push(root: Path) -> None:
    require_clean_worktree(root)
    repo = origin_identity(root)
    branch = default_branch(root, repo)
    for local_ref, local_oid, remote_ref, remote_oid in parse_push_lines(sys.stdin.read()):
        if local_ref != "(delete)":
            validate_ref_name(local_ref)
            run_git(root, "check-ref-format", local_ref)
        if remote_ref.startswith("refs/remotes/"):
            raise PolicyError("invalid pre-push ref line")
        run_git(root, "check-ref-format", remote_ref)
        validate_ref(remote_ref, branch, remote_oid, local_oid)
        if re.fullmatch(r"0{40}|0{64}", local_oid):
            if local_ref != "(delete)":
                raise PolicyError("zero local object id requires a deletion marker")
            continue
        current_tip_matches(root, remote_ref, local_oid)
        commits = commit_list(root, remote_oid, local_oid)
        for commit in commits:
            trailers = parse_git_trailers(root, run_git(root, "show", "-s", "--format=%B", commit))
            verify_issue(root, trailers.issue)
            verify_dependency_commit(root, commit, trailers)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    message = sub.add_parser("validate-commit-message")
    message.add_argument("path", type=Path)
    sub.add_parser("validate-push")
    issue = sub.add_parser("verify-issue")
    issue.add_argument("number", type=int)
    args = parser.parse_args(argv)
    try:
        root = Path.cwd()
        if args.command == "validate-commit-message":
            validate_message(args.path)
        elif args.command == "verify-issue":
            verify_issue(root, args.number)
        else:
            validate_push(root)
    except (OSError, PolicyError, subprocess.SubprocessError) as exc:
        return die(str(exc))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
