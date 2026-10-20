#!/usr/bin/env python3
"""Issue #9 public-release-gated cleanup CLI."""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

from cleanup_io import (
    CleanupIOError, ancestor, bound_branches, current_branch, default_branch,
    git, live_branch_commit, local_branches, local_tag_commit,
    origin_repo, remote_branch_commit, remote_branches, remote_tag_commit,
    stash_present, verify_latest_release, verify_published_crates, verify_public_release, worktree_dirty,
)
from cleanup_policy import CleanupPlan, CleanupPolicyError, REGISTRY_TRUST_LIMITATION, ReleaseFacts, local_plan, remote_plan, require_tag


def report(plan: CleanupPlan) -> None:
    print(json.dumps({"decisions": [item.__dict__ for item in plan.decisions], "residual": plan.residual, "registry_trust_limitation": REGISTRY_TRUST_LIMITATION}, ensure_ascii=True))


def facts(root: Path, tag: str, *, sync: bool) -> ReleaseFacts:
    tag = require_tag(tag)
    repo = origin_repo(root)
    verify_public_release(repo, tag)
    verify_published_crates(tag)
    branch = default_branch(repo, root)
    if sync:
        fetch_graph(root, tag)
    local_commit = local_tag_commit(root, tag)
    remote_commit = remote_tag_commit(root, tag)
    if local_commit != remote_commit:
        raise CleanupIOError("local and origin tag commits do not match")
    default_commit = remote_branch_commit(root, branch)
    if default_commit != live_branch_commit(root, branch):
        raise CleanupIOError("origin/default graph is stale; synchronize before cleanup")
    return ReleaseFacts(repo, tag, local_commit, branch, default_commit)


def run_remote(root: Path, tag: str, dry_run: bool) -> int:
    release = facts(root, tag, sync=not dry_run)
    branches = remote_branches(root)
    plan = remote_plan(branches, release.tag_commit, release.default_branch, release.default_commit, lambda child, parent: ancestor(root, child, parent))
    report(plan)
    if dry_run:
        return 2 if plan.residual else 0
    expected_refs = dict(branches)
    for decision in plan.decisions:
        if decision.action == "delete":
            expected = expected_refs.get(decision.name)
            if expected is None or live_branch_commit(root, decision.name) != expected:
                raise CleanupIOError(f"remote branch advanced before deletion: {decision.name}")
            git(root, "push", f"--force-with-lease=refs/heads/{decision.name}:{expected}", "origin", f":refs/heads/{decision.name}")
    return 2 if plan.residual else 0


def run_local(root: Path, tag: str, dry_run: bool) -> int:
    if worktree_dirty(root) or stash_present(root):
        raise CleanupIOError("local cleanup requires a clean worktree and empty stash")
    original = current_branch(root)
    release = facts(root, tag, sync=not dry_run)
    if dry_run:
        branches = [(name, "yes" if ancestor(root, commit, release.tag_commit) and ancestor(root, commit, release.default_commit) else "no") for name, commit in local_branches(root)]
        plan = local_plan(branches, bound_branches(root), release.default_branch, original)
        report(plan)
        return 2 if plan.residual else 0
    git(root, "switch", release.default_branch)
    env = os.environ.copy()
    env["KLE_RELEASE_CLEANUP_INTERNAL"] = "1"
    git(root, "pull", "--ff-only", "origin", release.default_branch, env=env)
    release = facts(root, tag, sync=False)
    head = git(root, "rev-parse", "HEAD").strip()
    if head != release.default_commit or current_branch(root) != release.default_branch:
        raise CleanupIOError("local HEAD is not the freshly verified origin/default commit")
    default_commit = release.default_commit
    branches = [(name, "yes" if ancestor(root, commit, release.tag_commit) and ancestor(root, commit, default_commit) else "no") for name, commit in local_branches(root)]
    plan = local_plan(branches, bound_branches(root), release.default_branch, current_branch(root))
    report(plan)
    expected_refs = dict(local_branches(root))
    for decision in plan.decisions:
        if decision.action == "delete":
            if worktree_dirty(root) or stash_present(root):
                raise CleanupIOError("local worktree or stash changed before branch deletion")
            if current_branch(root) != release.default_branch:
                raise CleanupIOError("current branch changed before branch deletion")
            if decision.name in bound_branches(root):
                raise CleanupIOError(f"local branch became worktree-bound before deletion: {decision.name}")
            expected = expected_refs[decision.name]
            current_tip = dict(local_branches(root)).get(decision.name)
            if current_tip != expected:
                raise CleanupIOError(f"local branch advanced before deletion: {decision.name}")
            if not ancestor(root, expected, release.tag_commit) or not ancestor(root, expected, release.default_commit):
                raise CleanupIOError(f"local branch ancestry changed before deletion: {decision.name}")
            git(root, "branch", "-d", decision.name)
    return 2 if plan.residual else 0


def run_post_merge(root: Path) -> int:
    repo = origin_repo(root)
    tag, raw = verify_latest_release(repo)
    if tag is None or raw is None:
        print("release-cleanup: no public latest release; no-op")
        return 0
    return run_local(root, tag, False)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("remote", "local", "verify-release"):
        item = sub.add_parser(name)
        item.add_argument("--tag", required=True)
        item.add_argument("--dry-run", action="store_true")
    sub.add_parser("post-merge")
    args = parser.parse_args(argv)
    root = Path.cwd()
    try:
        if args.command == "remote":
            return run_remote(root, args.tag, args.dry_run)
        if args.command == "local":
            return run_local(root, args.tag, args.dry_run)
        if args.command == "verify-release":
            facts(root, args.tag, sync=False)
            print(f"verified public release {args.tag}")
            return 0
        return run_post_merge(root)
    except (CleanupIOError, CleanupPolicyError, OSError) as exc:
        print(f"release-cleanup: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
