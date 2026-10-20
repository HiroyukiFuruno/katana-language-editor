"""Cargo manifest dependency normalization for the governance guard."""

from __future__ import annotations

import tomllib
from typing import TypeAlias

from policy import PolicyError

DependencyValue: TypeAlias = tuple[tuple[str, ...], tuple[tuple[str, str], ...]]
DEPENDENCY_TABLES = {"dependencies", "dev-dependencies", "build-dependencies"}


def _table_is_dependency(path: tuple[str, ...]) -> bool:
    return bool(path) and (path == ("workspace", "dependencies") or len(path) == 1 and path[-1] in DEPENDENCY_TABLES or len(path) == 3 and path[0] == "target" and path[-1] in DEPENDENCY_TABLES)


def dependency_entries(raw: bytes | None, workspace: dict[str, object] | None = None) -> dict[tuple[str, ...], DependencyValue]:
    if raw is None:
        return {}
    try:
        data = tomllib.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as exc:
        raise PolicyError("malformed Cargo.toml dependency manifest") from exc
    workspace = workspace or {}
    entries: dict[tuple[str, ...], DependencyValue] = {}
    for path, value in _walk_tables(data):
        if not _table_is_dependency(path) or not isinstance(value, dict):
            continue
        for alias, spec in value.items():
            package, normalized = _resolve(str(alias), spec, workspace, set())
            if package.startswith("katana-"):
                entries[path + (str(alias),)] = (path, tuple(sorted(normalized.items())))
    return entries


def _walk_tables(value: object, path: tuple[str, ...] = ()) -> list[tuple[tuple[str, ...], object]]:
    if not isinstance(value, dict):
        return []
    result = [(path, value)]
    for key, child in value.items():
        result.extend(_walk_tables(child, path + (str(key),)))
    return result


def _resolve(alias: str, spec: object, workspace: dict[str, object], seen: set[str]) -> tuple[str, dict[str, str]]:
    if isinstance(spec, str):
        return alias, {"package": alias, "version": spec}
    if not isinstance(spec, dict):
        raise PolicyError(f"invalid dependency specification for {alias}")
    if spec.get("workspace") is True:
        if alias in seen or alias not in workspace:
            raise PolicyError(f"invalid workspace dependency inheritance for {alias}")
        package, root = _resolve(alias, workspace[alias], workspace, seen | {alias})
        member = {str(key): repr(value) for key, value in spec.items() if key not in {"workspace", "package"}}
        if "package" in spec:
            package = _package_name(alias, spec["package"])
        return package, {"package": package, **{f"root:{key}": value for key, value in root.items() if key != "package"}, **{f"member:{key}": value for key, value in member.items()}}
    package = _package_name(alias, spec.get("package", alias))
    return package, {"package": package, **{str(key): repr(value) for key, value in spec.items()}}


def _package_name(alias: str, value: object) -> str:
    if not isinstance(value, str) or not value:
        raise PolicyError(f"invalid package identity for {alias}")
    return value


def dependency_changed(before: bytes | None, after: bytes | None, *, before_workspace: bytes | dict[str, object] | None = None, after_workspace: bytes | dict[str, object] | None = None) -> bool:
    def workspace(raw: bytes | dict[str, object] | None) -> dict[str, object]:
        if raw is None:
            return {}
        if isinstance(raw, dict):
            return raw
        try:
            data = tomllib.loads(raw.decode("utf-8"))
        except (UnicodeDecodeError, tomllib.TOMLDecodeError) as exc:
            raise PolicyError("malformed workspace Cargo.toml") from exc
        value = data.get("workspace", {})
        return value.get("dependencies", {}) if isinstance(value, dict) and isinstance(value.get("dependencies", {}), dict) else {}

    return dependency_entries(before, workspace(before_workspace)) != dependency_entries(after, workspace(after_workspace))
