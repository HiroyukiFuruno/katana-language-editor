"""Authenticate fixed epaint registry source members by package identity only."""

import hashlib
import io
from pathlib import Path
import tarfile
from urllib.parse import unquote, urlparse

from egui_definition_probe import _authenticated_source, _local_file_uri_path, _path_from_uri


VERSION = "0.36.1"
ARCHIVE_SHA256 = "11863a6b2b7823010c1090ddf4706947bdda46766742def0673195cbf7ff4a73"
PACKAGE = "epaint"
CARGO_REGISTRY_SRC = Path.home() / ".cargo" / "registry" / "src"


def _member_from_uri(uri):
    parsed = urlparse(uri)
    decoded_path = unquote(parsed.path)
    raw_path = Path(_local_file_uri_path(decoded_path))
    if (parsed.scheme != "file" or parsed.netloc not in ("", "localhost")
            or any(part in (".", "..") for part in decoded_path.split("/"))):
        return None
    resolved_path = raw_path.resolve(strict=False)
    try:
        relative = resolved_path.relative_to(CARGO_REGISTRY_SRC.resolve())
    except ValueError:
        return None
    parts = relative.parts
    if len(parts) < 4 or parts[1:3] != (f"{PACKAGE}-{VERSION}", "src") or not parts[0] or raw_path.suffix != ".rs":
        return None
    return parts[0], f"{PACKAGE}-{VERSION}/src/" + "/".join(parts[3:]), resolved_path


def _archive_member(archive_bytes, member):
    with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode="r:*") as source:
        members = [entry for entry in source.getmembers() if entry.name == member]
        if len(members) != 1 or not members[0].isreg():
            raise ValueError("epaint source tar member is missing or duplicated")
        extracted = source.extractfile(members[0])
        if extracted is None:
            raise ValueError("epaint source tar member is not readable")
        return extracted.read()


def authenticated_epaint_member(uri):
    """Return matching local source bytes only after archive and member verification."""
    candidate = _member_from_uri(uri)
    if candidate is None:
        return None
    registry_id, member, path = candidate
    archive = CARGO_REGISTRY_SRC.parent / "cache" / registry_id / f"{PACKAGE}-{VERSION}.crate"
    if not archive.is_file() or hashlib.sha256(archive.read_bytes()).hexdigest() != ARCHIVE_SHA256:
        raise ValueError("epaint crate archive SHA-256 mismatch")
    data = _archive_member(archive.read_bytes(), member)
    if data != path.read_bytes():
        raise ValueError("authenticated epaint source differs from the archive")
    return path, data


def authenticated_egui_member(uri):
    """Preserve the existing egui member identity check for hierarchy targets."""
    try:
        path = _path_from_uri(uri)
        relative = path.relative_to(Path.home() / ".cargo" / "registry" / "src")
    except ValueError:
        return None
    parts = relative.parts
    if (len(parts) < 4 or parts[1:3] != ("egui-0.36.1", "src") or path.suffix != ".rs"
            or not parts[0] or any(part in ("", ".", "..") for part in parts[3:])):
        return None
    member = "/".join(parts[1:])
    return _authenticated_source(uri, member, Path(member), "call-hierarchy target")
