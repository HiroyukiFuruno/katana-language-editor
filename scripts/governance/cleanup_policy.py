#!/usr/bin/env python3
"""Pure release-cleanup facts and planning rules."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from typing import Callable


class CleanupPolicyError(ValueError):
    pass


TAG_RE = re.compile(r"^v[0-9]+\.[0-9]+\.[0-9]+$")
REGISTRY_TRUST_LIMITATION = "cargo info exit=0 verifies the requested crate@version resolution; cargo may use its local crates.io index/cache, so this is not an independent propagation-freshness proof"


@dataclass(frozen=True)
class ReleaseFacts:
    repo: str
    tag: str
    tag_commit: str
    default_branch: str
    default_commit: str


@dataclass(frozen=True)
class Decision:
    name: str
    action: str
    reason: str


@dataclass(frozen=True)
class CleanupPlan:
    decisions: tuple[Decision, ...]

    @property
    def residual(self) -> bool:
        return any(
            item.action == "retain"
            and item.name.startswith("release/")
            and item.reason != "protected default branch"
            for item in self.decisions
        )


def require_tag(tag: str) -> str:
    if not TAG_RE.fullmatch(tag):
        raise CleanupPolicyError("tag must be an exact vX.Y.Z value")
    return tag


def registry_version(tag: str) -> str:
    return require_tag(tag).removeprefix("v")


def parse_release_payload(raw: str, repo: str, tag: str) -> dict[str, str]:
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise CleanupPolicyError("malformed GitHub release JSON") from exc
    expected_url = f"https://github.com/{repo}/releases/tag/{tag}"
    if not isinstance(value, dict) or value.get("tag_name") != tag:
        raise CleanupPolicyError("GitHub release tag does not match --tag")
    if value.get("draft") is not False or value.get("prerelease") is not False:
        raise CleanupPolicyError("GitHub release must be published and non-prerelease")
    if not isinstance(value.get("published_at"), str) or not value["published_at"]:
        raise CleanupPolicyError("GitHub release has no published_at")
    if value.get("html_url") != expected_url:
        raise CleanupPolicyError("GitHub release URL does not match origin")
    return {"tag_name": tag, "published_at": value["published_at"], "html_url": expected_url}


def remote_plan(branches: list[tuple[str, str]], tag_commit: str, default_branch: str, default_commit: str, ancestor: Callable[[str, str], bool]) -> CleanupPlan:
    decisions = []
    for name, commit in branches:
        if name == default_branch:
            decisions.append(Decision(name, "retain", "protected default branch"))
        elif not name.startswith("release/"):
            decisions.append(Decision(name, "retain", "not a release/* branch"))
        elif ancestor(commit, tag_commit) and ancestor(commit, default_commit):
            decisions.append(Decision(name, "delete", "tip is an ancestor of public tag and origin/default"))
        else:
            decisions.append(Decision(name, "retain", "tip is not an ancestor of both verified commits"))
    return CleanupPlan(tuple(decisions))


def local_plan(branches: list[tuple[str, str]], bound: set[str], default_branch: str, current_branch: str | None) -> CleanupPlan:
    protected = {default_branch, "master", "main", current_branch}
    decisions = []
    for name, merged in branches:
        if name == default_branch:
            decisions.append(Decision(name, "retain", "protected default branch"))
        elif name in protected:
            decisions.append(Decision(name, "retain", "protected branch"))
        elif not name.startswith("release/"):
            decisions.append(Decision(name, "retain", "not a release/* branch"))
        elif name in bound:
            decisions.append(Decision(name, "retain", "branch is bound to another worktree"))
        elif merged != "yes":
            decisions.append(Decision(name, "retain", "branch is not merged into actual default"))
        else:
            decisions.append(Decision(name, "delete", "unbound merged release/* branch"))
    return CleanupPlan(tuple(decisions))
