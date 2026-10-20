#!/usr/bin/env python3
"""Bounded, diagnostic-only TextBuffer definition checks for an initialized LSP client."""

import hashlib
from pathlib import Path
import re

from egui_definition_probe import _authenticated_builder, _authenticated_source, _location, _position, _range


TEXT_BUFFER_MEMBER = "egui-0.36.1/src/widgets/text_edit/text_buffer.rs"
TEXT_BUFFER_SOURCE_SUFFIX = Path(TEXT_BUFFER_MEMBER)
BUILDER_CALL = "text.delete_previous_char(cursor)"
TEXT_BUFFER_EDGES = (
    ("delete_selected_ccursor_range", "self.delete_selected_ccursor_range([min_ccursor, max_ccursor])"),
    ("delete_char_range", "self.delete_char_range(min.index..max.index)"),
)


def _locations(result):
    if not isinstance(result, list) or not result or not all(isinstance(item, dict) for item in result):
        raise ValueError("edge definition must return one or more locations")
    locations = []
    for item in result:
        uri = item.get("uri")
        span = item.get("range")
        if not isinstance(uri, str) or not isinstance(span, dict):
            raise ValueError("edge definition location is malformed")
        locations.append((uri, span))
    return locations


def _symbol_ranges(text, name):
    matches = [match.start() + len("    fn ")
               for match in re.finditer(rf"^    fn {re.escape(name)}\(", text, re.MULTILINE)]
    return [_range(text, offset, offset + len(name)) for offset in matches]


def _symbol_range(text, name):
    matches = _symbol_ranges(text, name)
    if len(matches) != 1:
        raise ValueError(f"ambiguous definition anchor: {name}")
    return matches[0]


def _request(client, uri, text, name, anchor, expected_range, expected_uri, resolution_kind):
    owner = "    fn delete_previous_char("
    if text.count(owner) != 1:
        raise ValueError("ambiguous owner: delete_previous_char")
    start = text.index(owner)
    following = re.search(r"^    fn ", text[start + len(owner):], re.MULTILINE)
    end = start + len(owner) + following.start() if following else len(text)
    body = text[start:end]
    if body.count(anchor) != 1:
        raise ValueError(f"ambiguous query anchor: {name}")
    query_offset = start + body.index(anchor) + anchor.index(name)
    position = _position(text, query_offset)
    results = client.request("textDocument/definition",
                             dict(textDocument={"uri": uri}, position=position))
    locations = _locations(results)
    if len(locations) != 1 or locations[0] != (expected_uri, expected_range):
        raise ValueError(f"edge {name} did not resolve to its exact authenticated symbol")
    return dict(name=name, source_sha256=hashlib.sha256(text.encode("utf-8")).hexdigest(),
                query_position=position, results=results, resolution_kind=resolution_kind,
                semantic_complete=False)


def _dynamic_dispatch_request(client, uri, text):
    name, anchor = TEXT_BUFFER_EDGES[-1]
    if text.count(anchor) != 1:
        raise ValueError(f"ambiguous query anchor: {name}")
    position = _position(text, text.index(anchor) + anchor.index(name))
    results = client.request("textDocument/definition", dict(textDocument={"uri": uri}, position=position))
    locations = _locations(results)
    expected_ranges = _symbol_ranges(text, name)
    if not expected_ranges or any(location[0] != uri or location[1] not in expected_ranges for location in locations):
        raise ValueError("edge delete_char_range did not resolve to an authenticated definition identifier")
    return dict(name=name, source_sha256=hashlib.sha256(text.encode("utf-8")).hexdigest(),
                query_position=position, results=results,
                resolution_kind="unresolved_dynamic_dispatch", semantic_complete=False)


def follow_text_buffer(client, show_result):
    """Follow builder deletion into authenticated TextBuffer defaults without dispatch guesses."""
    builder_uri, show_range = _location(show_result)
    builder, builder_data = _authenticated_builder(builder_uri)
    if builder_uri != builder.as_uri():
        raise ValueError("show definition URI is not the authenticated builder source URI")
    builder_text = builder_data.decode("utf-8")
    if builder_text.count("fn show") != 1:
        raise ValueError("ambiguous definition anchor: show")
    show_start = builder_text.index("fn show") + len("fn ")
    if show_range != _range(builder_text, show_start, show_start + len("show")):
        raise ValueError("show definition range is not the exact authenticated show symbol")
    if builder_text.count(BUILDER_CALL) != 1:
        raise ValueError("ambiguous query anchor: delete_previous_char")

    builder_position = _position(builder_text, builder_text.index(BUILDER_CALL) + len("text."))
    builder_results = client.request("textDocument/definition", dict(
        textDocument={"uri": builder_uri}, position=builder_position))
    builder_locations = _locations(builder_results)
    if len(builder_locations) != 1:
        raise ValueError("edge delete_previous_char must resolve to exactly one location")
    text_buffer_uri, text_buffer_range = builder_locations[0]
    text_buffer, text_buffer_data = _authenticated_source(
        text_buffer_uri, TEXT_BUFFER_MEMBER, TEXT_BUFFER_SOURCE_SUFFIX, "text_buffer")
    text_buffer_text = text_buffer_data.decode("utf-8")
    if text_buffer_uri != text_buffer.as_uri():
        raise ValueError("edge delete_previous_char URI is not the authenticated text_buffer source URI")
    delete_previous_range = _symbol_range(text_buffer_text, "delete_previous_char")
    if text_buffer_range != delete_previous_range:
        raise ValueError("edge delete_previous_char did not resolve to its exact authenticated symbol")
    client.send(dict(method="textDocument/didOpen", params={"textDocument": {
        "uri": text_buffer_uri, "languageId": "rust", "version": 1, "text": text_buffer_text}}))
    edges = [dict(name="delete_previous_char", source_sha256=hashlib.sha256(builder_data).hexdigest(),
                  query_position=builder_position, results=builder_results,
                  resolution_kind="lsp_default_method", semantic_complete=False)]
    for name, anchor in TEXT_BUFFER_EDGES:
        if name == "delete_char_range":
            edges.append(_dynamic_dispatch_request(client, text_buffer_uri, text_buffer_text))
        else:
            edges.append(_request(client, text_buffer_uri, text_buffer_text, name, anchor,
                                  _symbol_range(text_buffer_text, name), text_buffer_uri,
                                  "lsp_default_method"))
    return edges
