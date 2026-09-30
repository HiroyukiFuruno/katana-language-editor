#!/usr/bin/env python3
"""Bounded call-hierarchy evidence for authenticated egui cursor-range roots."""
import hashlib
from cursor_definition_probe import CURSOR_RANGE_MEMBER, CURSOR_RANGE_SOURCE_SUFFIX, _symbol_range
from egui_definition_probe import _authenticated_source, _range
from epaint_source_probe import authenticated_egui_member, authenticated_epaint_member
ROOTS = ("on_event", "on_key_press", "move_single_cursor")
FUNCTION_OR_METHOD_KINDS = {6, 12}
def _point(value, label):
    if not isinstance(value, dict):
        raise ValueError(f"{label} position is malformed")
    line, character = value.get("line"), value.get("character")
    if (not isinstance(line, int) or isinstance(line, bool) or line < 0
            or not isinstance(character, int) or isinstance(character, bool) or character < 0):
        raise ValueError(f"{label} position is malformed")
    return line, character
def _bounds(span, label):
    if not isinstance(span, dict):
        raise ValueError(f"{label} range is malformed")
    start, end = _point(span.get("start"), label), _point(span.get("end"), label)
    if start > end:
        raise ValueError(f"{label} range is malformed")
    return start, end
def _contains(owner, child, label, nonempty=False):
    owner_start, owner_end = _bounds(owner, label)
    child_start, child_end = _bounds(child, label)
    if (nonempty and child_start == child_end) or not (owner_start <= child_start and child_end <= owner_end):
        raise ValueError(f"{label} range is outside its owner")
def _utf16_offset(text, position, label):
    line, character = _point(position, label)
    lines = _lf_lines(text)
    if line >= len(lines):
        raise ValueError(f"{label} position is outside authenticated source")
    content, line_offset = lines[line]
    used = 0
    for index, scalar in enumerate(content):
        width = len(scalar.encode("utf-16-le")) // 2
        if used == character:
            return line_offset + index
        if used < character < used + width:
            raise ValueError(f"{label} position splits a UTF-16 code point")
        used += width
    if used == character:
        return line_offset + len(content)
    raise ValueError(f"{label} position is outside authenticated source")
def _lf_lines(text):
    """LSP lines are LF-delimited, with CR excluded only from a CRLF terminator."""
    lines = []
    offset = 0
    while True:
        newline = text.find("\n", offset)
        if newline < 0:
            lines.append((text[offset:], offset))
            return lines
        end = newline - 1 if newline > offset and text[newline - 1] == "\r" else newline
        lines.append((text[offset:end], offset))
        offset = newline + 1
def _excerpt(text, span, label):
    start, end = _bounds(span, label)
    start_offset = _utf16_offset(text, {"line": start[0], "character": start[1]}, label)
    end_offset = _utf16_offset(text, {"line": end[0], "character": end[1]}, label)
    if start_offset >= end_offset:
        raise ValueError(f"{label} range is empty")
    return text[start_offset:end_offset]
def _location(result, name):
    if not isinstance(result, list):
        raise ValueError(f"root {name} definition result is malformed")
    if len(result) != 1 or not isinstance(result[0], dict):
        raise ValueError(f"root {name} definition must contain exactly one location")
    uri, span = result[0].get("uri"), result[0].get("range")
    if not isinstance(uri, str) or not isinstance(span, dict):
        raise ValueError(f"root {name} definition location is malformed")
    return uri, span
def _roots(cursor_edges):
    if not isinstance(cursor_edges, list):
        raise ValueError("cursor definition result is malformed")
    sources = {}
    for edge in cursor_edges:
        if not isinstance(edge, dict):
            raise ValueError("cursor definition edge is malformed")
        name = edge.get("name")
        if not isinstance(name, str):
            raise ValueError("cursor definition edge name is malformed")
        if name == "cursor_range.on_event":
            name = "on_event"
        elif name.startswith("move_single_cursor"):
            name = "move_single_cursor"
        if name not in ROOTS:
            continue
        uri, span = _location(edge.get("result"), name)
        previous = sources.get(name)
        if previous is not None and previous != (uri, span):
            raise ValueError(f"root {name} has inconsistent definition results")
        sources[name] = (uri, span)
    if set(sources) != set(ROOTS):
        raise ValueError("cursor definition results do not contain the three fixed roots")
    return sources

def _item(item, name, uri, identifier, text, label):
    if not isinstance(item, dict):
        raise ValueError(f"{label} is malformed")
    if item.get("name") != name or item.get("uri") != uri or item.get("selectionRange") != identifier:
        raise ValueError(f"{label} does not match its authenticated root identifier")
    kind = item.get("kind")
    if (not isinstance(kind, int) or isinstance(kind, bool)
            or kind not in FUNCTION_OR_METHOD_KINDS):
        raise ValueError(f"{label} kind is not Function or Method")
    owner = item.get("range")
    _contains(owner, identifier, label, nonempty=True)
    _excerpt(text, owner, label)
    _excerpt(text, identifier, label)


def _target(item):
    if not isinstance(item, dict):
        raise ValueError("outgoing target is malformed")
    uri, name, span, selection = item.get("uri"), item.get("name"), item.get("range"), item.get("selectionRange")
    if not isinstance(uri, str) or not uri or not isinstance(name, str) or not name:
        raise ValueError("outgoing target is malformed")
    _contains(span, selection, "outgoing target", nonempty=True)
    authenticated = authenticated_egui_member(uri)
    if authenticated is not None:
        _, data = authenticated
        _excerpt(data.decode("utf-8"), span, "outgoing target")
        return dict(item=item, resolution_kind="authenticated_egui_member_source_identity_only",
                    target_sha256=hashlib.sha256(data).hexdigest(), target_name_verified=False)
    authenticated = authenticated_epaint_member(uri)
    if authenticated is None:
        return dict(item=item, resolution_kind="unresolved_external_boundary", target_package=None,
                    target_sha256=None)
    _, data = authenticated
    text = data.decode("utf-8")
    _excerpt(text, span, "outgoing target")
    return dict(item=item, resolution_kind="authenticated_epaint_member_source_identity_only",
                target_package="epaint", target_sha256=hashlib.sha256(data).hexdigest(),
                semantic_complete=False, target_name_verified=False)

def _outgoing(client, root, text):
    result = client.request("callHierarchy/outgoingCalls", {"item": root})
    if result is None:
        return dict(outcome="unresolved_null", calls=[])
    if not isinstance(result, list):
        raise ValueError("outgoing calls result is malformed")
    calls = []
    for call in result:
        if not isinstance(call, dict) or not isinstance(call.get("fromRanges"), list):
            raise ValueError("outgoing call is malformed")
        if not call["fromRanges"]:
            raise ValueError("outgoing call has no fromRanges evidence")
        from_ranges = sorted(call["fromRanges"], key=_range_sort_key)
        excerpts = []
        for span in from_ranges:
            _contains(root["range"], span, "outgoing fromRange", nonempty=True)
            excerpts.append(dict(range=span, text=_excerpt(text, span, "outgoing fromRange")))
        target = _target(call.get("to"))
        calls.append(dict(to=target["item"], fromRanges=from_ranges, from_range_excerpts=excerpts,
                          resolution_kind=target["resolution_kind"], target_sha256=target["target_sha256"],
                          target_package=target.get("target_package"),
                          target_name_verified=target.get("target_name_verified", False)))
    calls.sort(key=lambda call: (call["to"]["uri"], call["to"]["name"],
                                 _range_sort_key(call["to"]["selectionRange"]),
                                 tuple(_range_sort_key(span) for span in call["fromRanges"])))
    return dict(outcome="empty" if not calls else "calls", calls=calls)

def _range_sort_key(span):
    start, end = _bounds(span, "range sort key")
    return start[0], start[1], end[0], end[1]

def follow_cursor_call_hierarchy(client, cursor_edges):
    """Collect direct calls only; callers must record errors and never infer closure."""
    roots = _roots(cursor_edges)
    uri = next(uri for uri, _ in roots.values())
    if any(candidate_uri != uri for candidate_uri, _ in roots.values()):
        raise ValueError("fixed cursor roots do not share one authenticated URI")
    path, data = _authenticated_source(uri, CURSOR_RANGE_MEMBER, CURSOR_RANGE_SOURCE_SUFFIX, "cursor_range")
    if uri != path.as_uri():
        raise ValueError("fixed cursor root URI is not the authenticated cursor_range source URI")
    text = data.decode("utf-8")
    report_roots = []
    for name in ROOTS:
        root_uri, identifier = roots[name]
        if identifier != _symbol_range(text, name):
            raise ValueError(f"root {name} identifier is not the exact authenticated symbol")
        prepared = client.request("textDocument/prepareCallHierarchy", {
            "textDocument": {"uri": root_uri}, "position": identifier["start"]})
        if prepared is None:
            raise ValueError(f"prepareCallHierarchy returned null for root {name}")
        if not isinstance(prepared, list):
            raise ValueError(f"prepareCallHierarchy result is malformed for root {name}")
        if not prepared:
            raise ValueError(f"prepareCallHierarchy returned empty for root {name}")
        if len(prepared) != 1:
            raise ValueError(f"prepareCallHierarchy returned multiple items for root {name}")
        root = prepared[0]
        _item(root, name, root_uri, identifier, text, f"prepared root {name}")
        outgoing = _outgoing(client, root, text)
        report_roots.append(dict(name=name, uri=root_uri, identifier_range=identifier, root=root,
                                 source_sha256=hashlib.sha256(data).hexdigest(), outgoing=outgoing))
    return dict(diagnostic_only=True, semantic_complete=False, transitive_traversal_performed=False,
                roots=report_roots, limits=["direct outgoing calls only", "transitive traversal not performed"])
