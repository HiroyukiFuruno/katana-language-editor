#!/usr/bin/env python3
"""Pure policy functions for the series commit/push guard."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
OID_RE = re.compile(r"^[0-9a-fA-F]{40}$|^[0-9a-fA-F]{64}$")
ZERO_RE = re.compile(r"^(0{40}|0{64})$")
ISSUE_RE = re.compile(r"^#[1-9][0-9]*$")
REF_BAD_RE = re.compile(r"[\x00-\x20~^:?*\\\[]|\.\.|@\{|//|(?:^|/)(?:\.|[^/]*\.(?:/|$))|(?:^|/)[^/]*\.lock(?:/|$)")


class PolicyError(ValueError):
    """A fail-closed policy violation."""


@dataclass(frozen=True)
class Trailers:
    issue: int
    downstream_issue: int | None


def validate_parsed_trailers(lines: list[str]) -> Trailers:
    """Validate records emitted by Git's interpret-trailers command."""
    values: dict[str, list[str]] = {"Issue": [], "Downstream-Issue": []}
    for line in lines:
        key, separator, value = line.partition(":")
        if separator and key in values:
            values[key].append(value.strip())
    if len(values["Issue"]) != 1 or not ISSUE_RE.fullmatch(values["Issue"][0]):
        raise PolicyError("exactly one positive Issue: #N trailer is required")
    if len(values["Downstream-Issue"]) > 1:
        raise PolicyError("Downstream-Issue trailer must occur at most once")
    downstream = values["Downstream-Issue"]
    if downstream and not ISSUE_RE.fullmatch(downstream[0]):
        raise PolicyError("Downstream-Issue must use #N with a positive number")
    return Trailers(int(values["Issue"][0][1:]), int(downstream[0][1:]) if downstream else None)


def parse_issue_json(raw: str, *, expected_number: int | None = None, expected_repo: str | None = None) -> tuple[str, str]:
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise PolicyError(f"malformed GitHub issue JSON: {exc}") from exc
    if not isinstance(value, dict) or not isinstance(value.get("state"), str) or not isinstance(value.get("body"), str):
        raise PolicyError("GitHub issue JSON has malformed state or body")
    if "pull_request" in value:
        raise PolicyError("GitHub response is a pull request, not an issue")
    if expected_number is not None and (type(value.get("number")) is not int or value["number"] != expected_number):
        raise PolicyError("GitHub issue number does not match the requested issue")
    if expected_repo is not None and value.get("html_url") != f"https://github.com/{expected_repo}/issues/{expected_number}":
        raise PolicyError("GitHub issue URL does not match the repository and issue")
    body = value["body"]
    if not isinstance(body, str):
        raise PolicyError("GitHub issue JSON has a non-string body")
    return value["state"], body


def require_open_issue(raw: str, *, expected_number: int, expected_repo: str, metadata: bool = False) -> None:
    state, body = parse_issue_json(raw, expected_number=expected_number, expected_repo=expected_repo)
    if state != "open":
        raise PolicyError(f"GitHub issue is not open: {state!r}")
    if metadata:
        for name in ("Upstream-Release", "Dependency-Update", "Validation"):
            if not re.search(rf"(?im)^[ \t]*{re.escape(name)}:[ \t]*[^\r\n \t]", body):
                raise PolicyError(f"Downstream issue is missing nonempty {name} metadata")


def validate_ref_name(ref: str) -> None:
    """Reject obvious malformed refs; the CLI also delegates to Git's parser."""
    if not ref.startswith("refs/") or not ref[5:] or REF_BAD_RE.search(ref):
        raise PolicyError(f"invalid ref: {ref!r}")


def validate_ref(ref: str, default_branch: str, old_oid: str, new_oid: str) -> None:
    validate_ref_name(ref)
    if not OID_RE.fullmatch(old_oid) or not OID_RE.fullmatch(new_oid):
        raise PolicyError(f"invalid object id in ref update: {ref}")
    if len(old_oid) != len(new_oid):
        raise PolicyError(f"object id widths do not match: {ref}")
    deleting = bool(ZERO_RE.fullmatch(new_oid))
    if ref == f"refs/heads/{default_branch}":
        raise PolicyError("default branch may not be updated or deleted")
    if ref.startswith("refs/heads/"):
        branch = ref.removeprefix("refs/heads/")
        if deleting:
            return
        elif not branch.startswith("release/"):
            raise PolicyError("new and updated branches must use release/*")
    elif ref.startswith("refs/tags/"):
        if deleting:
            raise PolicyError("tag deletion is outside the Issue #9 core guard")
    else:
        raise PolicyError(f"unsupported ref namespace: {ref}")
