#!/usr/bin/env python3
"""Bounded, read-only egui definition checks for an initialized LSP client."""

import hashlib
import io
import os
from pathlib import Path
import tarfile
from urllib.parse import unquote, urlparse


VERSION = "0.36.1"
ARCHIVE_SHA256 = "c977ac91dfaa651633fd9722e4ce9ccb32cda4c748b89a5cb57e504036e37c13"
BUILDER_MEMBER = "egui-0.36.1/src/widgets/text_edit/builder.rs"
BUILDER_SOURCE_SUFFIX = Path(BUILDER_MEMBER)
# Kept for the existing bounded builder probe and its callers.
MEMBER = BUILDER_MEMBER
SOURCE_SUFFIX = BUILDER_SOURCE_SUFFIX
EDGES = (
    ("events", "new_cursor_range) = events(", "new_cursor_range) = "),
    ("check_for_mutating_key_press", "} => check_for_mutating_key_press(", "} => "),
)


def _location(result):
    if not isinstance(result, list) or len(result) != 1 or not isinstance(result[0], dict):
        raise ValueError("show definition must return exactly one location")
    location = result[0]
    uri = location.get("uri")
    span = location.get("range")
    if not isinstance(uri, str) or not isinstance(span, dict):
        raise ValueError("show definition location is malformed")
    return uri, span


def _path_from_uri(uri):
    parsed = urlparse(uri)
    if parsed.scheme != "file" or parsed.netloc not in ("", "localhost"):
        raise ValueError("show definition is not a local file URI")
    return Path(_local_file_uri_path(unquote(parsed.path))).resolve(strict=False)


def _local_file_uri_path(path):
    """Convert a file URI path without turning a Windows drive into a relative path."""
    if os.name == "nt" and len(path) >= 3 and path[0] == "/" and path[1].isalpha() and path[2] == ":":
        return path[1:]
    return path


def _authenticated_source(uri, member, source_suffix, label):
    path = _path_from_uri(uri)
    cargo_src = Path.home() / ".cargo" / "registry" / "src"
    try:
        relative = path.relative_to(cargo_src)
    except ValueError as error:
        raise ValueError("show definition is outside the cargo registry source cache") from error
    parts = relative.parts
    if len(parts) != len(source_suffix.parts) + 1 or parts[1:] != source_suffix.parts or not parts[0]:
        raise ValueError(f"show definition is not the egui 0.36.1 {label} source")
    registry_id = parts[0]
    archive = Path.home() / ".cargo" / "registry" / "cache" / registry_id / f"egui-{VERSION}.crate"
    if not archive.is_file():
        raise ValueError("matching egui crate archive is missing")
    archive_bytes = archive.read_bytes()
    archive_sha256 = hashlib.sha256(archive_bytes).hexdigest()
    if archive_sha256 != ARCHIVE_SHA256:
        raise ValueError("egui crate archive SHA-256 mismatch")
    with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode="r:*") as source:
        members = [entry for entry in source.getmembers() if entry.name == member]
        if len(members) != 1 or not members[0].isfile():
            raise ValueError(f"egui {label} tar member is missing or duplicated")
        extracted = source.extractfile(members[0])
        if extracted is None:
            raise ValueError(f"egui {label} tar member is not readable")
        data = extracted.read()
    if data != path.read_bytes():
        raise ValueError(f"authenticated egui {label} source differs from the archive")
    return path, data


def _authenticated_builder(uri):
    return _authenticated_source(uri, BUILDER_MEMBER, BUILDER_SOURCE_SUFFIX, "builder")


def _position(text, offset):
    prefix = text[:offset]
    return {"line": prefix.count("\n"),
            "character": len(prefix.rsplit("\n", 1)[-1].encode("utf-16-le")) // 2}


def _range(text, start, end):
    return {"start": _position(text, start), "end": _position(text, end)}


def _target(result):
    if not isinstance(result, list) or len(result) != 1 or not isinstance(result[0], dict):
        raise ValueError("edge definition must return exactly one location")
    location = result[0]
    uri = location.get("uri")
    span = location.get("range")
    if not isinstance(uri, str) or not isinstance(span, dict):
        raise ValueError("edge definition location is malformed")
    return uri, span


def follow_edges(client, show_result):
    """Verify egui's authenticated builder and follow its two bounded definitions."""
    show_uri, show_range = _location(show_result)
    builder, data = _authenticated_builder(show_uri)
    text = data.decode("utf-8")
    if show_uri != builder.as_uri():
        raise ValueError("show definition URI is not the authenticated source URI")
    show_start = text.index("fn show") + len("fn ")
    if show_range != _range(text, show_start, show_start + len("show")):
        raise ValueError("show definition range is not the exact show symbol")
    client.send(dict(method="textDocument/didOpen", params={"textDocument": {
        "uri": show_uri, "languageId": "rust", "version": 1, "text": text}}))
    source_sha256 = hashlib.sha256(data).hexdigest()
    edges = []
    for name, anchor, query_prefix in EDGES:
        if text.count(anchor) != 1:
            raise ValueError(f"ambiguous query anchor: {name}")
        query_offset = text.index(anchor) + len(query_prefix)
        position = _position(text, query_offset)
        result = client.request("textDocument/definition",
                                dict(textDocument={"uri": show_uri}, position=position))
        result_uri, result_range = _target(result)
        declaration = f"fn {name}("
        if text.count(declaration) != 1:
            raise ValueError(f"ambiguous definition anchor: {name}")
        symbol_start = text.index(declaration) + len("fn ")
        expected_range = _range(text, symbol_start, symbol_start + len(name))
        if result_uri != show_uri or result_range != expected_range:
            raise ValueError(f"edge {name} did not resolve to its exact authenticated symbol")
        edges.append(dict(name=name, source_sha256=source_sha256, position=position, result=result))
    return edges
