#!/usr/bin/env python3
"""Bounded, diagnostic-only cursor definition checks for an initialized LSP client."""

import hashlib
from pathlib import Path

from egui_definition_probe import _authenticated_builder, _authenticated_source, _position, _range


CURSOR_RANGE_MEMBER = "egui-0.36.1/src/text_selection/cursor_range.rs"
CURSOR_RANGE_SOURCE_SUFFIX = Path(CURSOR_RANGE_MEMBER)
BUILDER_CALL = "cursor_range.on_event(os, event, galley, id)"
ON_KEY_PRESS_CALL = "} => self.on_key_press(os, galley, modifiers, *key),"
MOVE_SINGLE_CURSOR_CALLS = (
    ("move_single_cursor_navigation", "| Key::End => {\n                move_single_cursor("),
    ("move_single_cursor_macos_control", "if os == OperatingSystem::Mac && modifiers.ctrl && !modifiers.shift =>\n            {\n                move_single_cursor("),
)


def _location(result, label):
    if not isinstance(result, list) or len(result) != 1 or not isinstance(result[0], dict):
        raise ValueError(f"edge {label} must resolve to exactly one location")
    location = result[0]
    uri = location.get("uri")
    span = location.get("range")
    if not isinstance(uri, str) or not isinstance(span, dict):
        raise ValueError(f"edge {label} definition location is malformed")
    return uri, span


def _symbol_range(text, name):
    declaration = f"fn {name}("
    if text.count(declaration) != 1:
        raise ValueError(f"ambiguous definition anchor: {name}")
    start = text.index(declaration) + len("fn ")
    return _range(text, start, start + len(name))


def _impl_range(text):
    anchor = "impl CCursorRange {"
    if text.count(anchor) != 1:
        raise ValueError("ambiguous definition anchor: impl CCursorRange")
    start = text.index(anchor) + len("impl ")
    return _range(text, start, start + len("CCursorRange"))


def _point(position):
    if not isinstance(position, dict):
        raise ValueError("document symbol range is malformed")
    line = position.get("line")
    character = position.get("character")
    if not isinstance(line, int) or isinstance(line, bool) or line < 0:
        raise ValueError("document symbol range is malformed")
    if not isinstance(character, int) or isinstance(character, bool) or character < 0:
        raise ValueError("document symbol range is malformed")
    return line, character


def _range_bounds(span):
    if not isinstance(span, dict):
        raise ValueError("document symbol range is malformed")
    start = _point(span.get("start"))
    end = _point(span.get("end"))
    if start > end:
        raise ValueError("document symbol range is malformed")
    return start, end


def _contains(span, position):
    start, end = _range_bounds(span)
    return start <= _point(position) < end


def _contains_range(owner, method):
    owner_start, owner_end = _range_bounds(owner)
    method_start, method_end = _range_bounds(method)
    if method_start >= method_end or not (owner_start <= method_start and method_end <= owner_end):
        raise ValueError("method range is outside CCursorRange impl owner")


def _document_symbol_range(symbol, expected, label):
    if not isinstance(symbol, dict) or symbol.get("selectionRange") != expected:
        raise ValueError(f"{label} selection range is not its exact authenticated identifier")
    span = symbol.get("range")
    if not _contains(span, expected["start"]) or not _contains(span, {
        "line": expected["end"]["line"], "character": expected["end"]["character"] - 1
    }):
        raise ValueError(f"{label} owner range is malformed")
    return span


def _method_owners(client, uri, text):
    symbols = client.request("textDocument/documentSymbol", dict(textDocument={"uri": uri}))
    if not isinstance(symbols, list):
        raise ValueError("cursor_range document symbols are malformed")
    owners = [symbol for symbol in symbols if isinstance(symbol, dict) and symbol.get("name") == "impl CCursorRange"]
    if len(owners) != 1:
        raise ValueError("cursor_range must have exactly one CCursorRange impl owner")
    owner = owners[0]
    owner_range = _document_symbol_range(owner, _impl_range(text), "CCursorRange impl")
    children = owner.get("children")
    if not isinstance(children, list):
        raise ValueError("CCursorRange impl owner is missing method symbols")
    ranges = {}
    for name in ("on_event", "on_key_press"):
        methods = [symbol for symbol in children if isinstance(symbol, dict) and symbol.get("name") == name]
        if len(methods) != 1:
            raise ValueError(f"CCursorRange impl must have exactly one {name} symbol")
        method_range = _document_symbol_range(methods[0], _symbol_range(text, name), name)
        _contains_range(owner_range, method_range)
        ranges[name] = method_range
    return ranges


def _request(client, uri, text, owner_range, name, anchor, query_symbol, expected_range):
    if text.count(anchor) != 1:
        raise ValueError(f"ambiguous query anchor: {name}")
    query_offset = text.index(anchor) + anchor.index(query_symbol)
    position = _position(text, query_offset)
    if not _contains(owner_range, position):
        raise ValueError(f"query anchor is outside structured owner: {name}")
    result = client.request("textDocument/definition", dict(textDocument={"uri": uri}, position=position))
    if _location(result, name) != (uri, expected_range):
        raise ValueError(f"edge {name} did not resolve to its exact authenticated symbol")
    return dict(name=name, source_sha256=hashlib.sha256(text.encode("utf-8")).hexdigest(),
                query_position=position, owner_range=owner_range, result=result, semantic_complete=False)


def follow_cursor_range(client, show_result):
    """Follow the builder opened by follow_edges through fixed cursor call sites."""
    builder_uri, show_range = _location(show_result, "show")
    builder, builder_data = _authenticated_builder(builder_uri)
    if builder_uri != builder.as_uri():
        raise ValueError("show definition URI is not the authenticated builder source URI")
    builder_text = builder_data.decode("utf-8")
    if show_range != _symbol_range(builder_text, "show"):
        raise ValueError("show definition range is not the exact authenticated show symbol")
    if builder_text.count(BUILDER_CALL) != 1:
        raise ValueError("ambiguous query anchor: cursor_range.on_event")
    builder_position = _position(builder_text, builder_text.index(BUILDER_CALL) + len("cursor_range."))
    builder_result = client.request("textDocument/definition", dict(
        textDocument={"uri": builder_uri}, position=builder_position))
    cursor_uri, cursor_range = _location(builder_result, "cursor_range.on_event")
    cursor_path, cursor_data = _authenticated_source(
        cursor_uri, CURSOR_RANGE_MEMBER, CURSOR_RANGE_SOURCE_SUFFIX, "cursor_range")
    if cursor_uri != cursor_path.as_uri():
        raise ValueError("edge cursor_range.on_event URI is not the authenticated cursor_range source URI")
    cursor_text = cursor_data.decode("utf-8")
    if cursor_range != _symbol_range(cursor_text, "on_event"):
        raise ValueError("edge cursor_range.on_event did not resolve to its exact authenticated symbol")
    client.send(dict(method="textDocument/didOpen", params={"textDocument": {
        "uri": cursor_uri, "languageId": "rust", "version": 1, "text": cursor_text}}))
    owners = _method_owners(client, cursor_uri, cursor_text)
    edges = [dict(name="cursor_range.on_event", source_sha256=hashlib.sha256(builder_data).hexdigest(),
                  query_position=builder_position, result=builder_result, semantic_complete=False)]
    edges.append(_request(client, cursor_uri, cursor_text, owners["on_event"], "on_key_press", ON_KEY_PRESS_CALL,
                          "on_key_press",
                          _symbol_range(cursor_text, "on_key_press")))
    move_range = _symbol_range(cursor_text, "move_single_cursor")
    for name, anchor in MOVE_SINGLE_CURSOR_CALLS:
        edges.append(_request(client, cursor_uri, cursor_text, owners["on_key_press"], name, anchor,
                              "move_single_cursor", move_range))
    return edges
