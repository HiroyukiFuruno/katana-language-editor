import importlib.util
from pathlib import Path
import sys
import unittest
from unittest import mock


DIRECTORY = Path(__file__).parent
sys.path.insert(0, str(DIRECTORY))
SPEC = importlib.util.spec_from_file_location(
    "cursor_call_hierarchy_probe", DIRECTORY / "cursor_call_hierarchy_probe.py")
probe = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(probe)


class FakeClient:
    def __init__(self, results):
        self.results = iter(results)
        self.requests = []

    def request(self, method, params):
        self.requests.append((method, params))
        return next(self.results)


def _text():
    return ("impl CCursorRange {\n"
            "pub fn on_event() { 日本😀⭐️ self.on_key_press(); }\n"
            "pub fn on_key_press() { move_single_cursor(); }\n"
            "fn move_single_cursor() {}\n"
            "}\n")


def _root(text, name):
    identifier = probe._symbol_range(text, name)
    line_start = text.rfind("\n", 0, text.index(f"fn {name}(")) + 1
    line_end = text.index("\n", text.index(f"fn {name}("))
    return {"name": name, "kind": 12, "uri": "file:///tmp/cursor_range.rs",
            "range": probe._range(text, line_start, line_end), "selectionRange": identifier}


def _edges(text):
    uri = "file:///tmp/cursor_range.rs"
    return [
        {"name": "cursor_range.on_event", "result": [{"uri": uri, "range": probe._symbol_range(text, "on_event")} ]},
        {"name": "on_key_press", "result": [{"uri": uri, "range": probe._symbol_range(text, "on_key_press")} ]},
        {"name": "move_single_cursor_navigation", "result": [{"uri": uri, "range": probe._symbol_range(text, "move_single_cursor")} ]},
        {"name": "move_single_cursor_macos_control", "result": [{"uri": uri, "range": probe._symbol_range(text, "move_single_cursor")} ]},
    ]


class CursorCallHierarchyTests(unittest.TestCase):
    def test_utf16_excerpt_accepts_japanese_vs16_and_non_bmp_but_rejects_split_surrogate(self):
        text = "日本😀⭐️x"
        self.assertEqual(probe._excerpt(text, probe._range(text, 2, 5), "fixture"), "😀⭐️")
        with self.assertRaisesRegex(ValueError, "splits a UTF-16 code point"):
            probe._excerpt(text, {"start": {"line": 0, "character": 2},
                                  "end": {"line": 0, "character": 3}}, "fixture")

    def test_utf16_positions_use_only_lf_and_crlf_line_boundaries(self):
        text = "a\r\n日😀\v⭐️\n"
        self.assertEqual(probe._excerpt(text, {"start": {"line": 1, "character": 1},
                                               "end": {"line": 1, "character": 6}}, "fixture"), "😀\v⭐️")
        self.assertEqual(probe._utf16_offset(text, {"line": 2, "character": 0}, "fixture"), len(text))

    def test_collects_exact_three_roots_once_and_keeps_unknown_boundaries(self):
        text = _text()
        roots = [_root(text, name) for name in probe.ROOTS]
        from_range = probe._range(text, text.index("self.on_key_press"),
                                  text.index("self.on_key_press") + len("self.on_key_press"))
        external = {"name": "String", "kind": 23, "uri": "file:///rustc/std/string.rs",
                    "range": {"start": {"line": 0, "character": 0}, "end": {"line": 2, "character": 0}},
                    "selectionRange": {"start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 6}}}
        responses = [[roots[0]], [{"to": external, "fromRanges": [from_range]}], [roots[1]], [], [roots[2]], None]
        client = FakeClient(responses)
        with mock.patch.object(probe, "_authenticated_source", return_value=(Path("/tmp/cursor_range.rs"), text.encode())), \
             mock.patch.object(probe, "authenticated_epaint_member", return_value=None):
            report = probe.follow_cursor_call_hierarchy(client, _edges(text))
        self.assertFalse(report["semantic_complete"])
        self.assertFalse(report["transitive_traversal_performed"])
        self.assertEqual([root["name"] for root in report["roots"]], list(probe.ROOTS))
        self.assertEqual(report["roots"][0]["outgoing"]["calls"][0]["resolution_kind"],
                         "unresolved_external_boundary")
        self.assertEqual(report["roots"][0]["outgoing"]["calls"][0]["from_range_excerpts"][0]["text"],
                         "self.on_key_press")
        self.assertEqual(report["roots"][2]["outgoing"]["outcome"], "unresolved_null")
        self.assertEqual(sum(method == "textDocument/prepareCallHierarchy" for method, _ in client.requests), 3)

    def test_rejects_null_empty_and_malformed_prepare_results_separately(self):
        text = _text()
        for result, message in ((None, "returned null"), ([], "returned empty"), ({}, "malformed")):
            with self.subTest(result=result), \
                 mock.patch.object(probe, "_authenticated_source", return_value=(Path("/tmp/cursor_range.rs"), text.encode())), \
                 self.assertRaisesRegex(ValueError, message):
                probe.follow_cursor_call_hierarchy(FakeClient([result]), _edges(text))

    def test_rejects_inconsistent_duplicate_root_and_out_of_root_from_range(self):
        text = _text()
        inconsistent = _edges(text)
        inconsistent[-1]["result"][0]["range"] = probe._range(text, 0, 1)
        with self.assertRaisesRegex(ValueError, "inconsistent"):
            probe.follow_cursor_call_hierarchy(FakeClient([]), inconsistent)
        root = _root(text, "on_event")
        outside = probe._range(text, text.index("move_single_cursor();"),
                               text.index("move_single_cursor();") + len("move_single_cursor"))
        target = {"name": "target", "uri": "file:///tmp/unknown.rs", "range": {
            "start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 6}},
            "selectionRange": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 6}}}
        with mock.patch.object(probe, "_authenticated_source", return_value=(Path("/tmp/cursor_range.rs"), text.encode())), \
             mock.patch.object(probe, "authenticated_epaint_member", return_value=None), \
             self.assertRaisesRegex(ValueError, "fromRange.*outside"):
            probe.follow_cursor_call_hierarchy(FakeClient([
                [root], [{"to": target, "fromRanges": [outside]}]
            ]), _edges(text))

    def test_rejects_inconsistent_target_ranges_and_modified_authenticated_member(self):
        malformed_target = {"name": "x", "uri": "file:///tmp/unknown.rs", "range": {
            "start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 1}},
            "selectionRange": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}}
        with self.assertRaisesRegex(ValueError, "outgoing target range is outside"):
            probe._target(malformed_target)
        with mock.patch.object(probe, "authenticated_epaint_member", side_effect=ValueError("source differs from the archive")), \
             self.assertRaisesRegex(ValueError, "differs from the archive"):
            probe._target({"name": "x", "uri": "file:///tmp/egui.rs", "range": {
                "start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}},
                "selectionRange": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}})

    def test_rejects_non_function_root_and_empty_from_ranges(self):
        text = _text()
        bad_root = _root(text, "on_event")
        bad_root["kind"] = 1
        with mock.patch.object(probe, "_authenticated_source", return_value=(Path("/tmp/cursor_range.rs"), text.encode())), \
             self.assertRaisesRegex(ValueError, "Function or Method"):
            probe.follow_cursor_call_hierarchy(FakeClient([[bad_root]]), _edges(text))
        target = {"name": "target", "uri": "file:///tmp/unknown.rs", "range": {
            "start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 6}},
            "selectionRange": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 6}}}
        with mock.patch.object(probe, "_authenticated_source", return_value=(Path("/tmp/cursor_range.rs"), text.encode())), \
             self.assertRaisesRegex(ValueError, "no fromRanges evidence"):
            probe.follow_cursor_call_hierarchy(FakeClient([[_root(text, "on_event")], [{"to": target, "fromRanges": []}]]),
                                               _edges(text))

    def test_rejects_float_root_kind_and_empty_target_identity(self):
        text = _text()
        floating_kind = _root(text, "on_event")
        floating_kind["kind"] = 6.0
        with mock.patch.object(probe, "_authenticated_source", return_value=(Path("/tmp/cursor_range.rs"), text.encode())), \
             self.assertRaisesRegex(ValueError, "Function or Method"):
            probe.follow_cursor_call_hierarchy(FakeClient([[floating_kind]]), _edges(text))
        for target in ({"name": "", "uri": "file:///tmp/a.rs"}, {"name": "x", "uri": ""}):
            with self.subTest(target=target), self.assertRaisesRegex(ValueError, "outgoing target is malformed"):
                probe._target(target)

    def test_preserves_egui_and_adds_epaint_source_classification(self):
        target = {"name": "LayoutJob", "uri": "file:///tmp/target.rs", "range": {
            "start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 6}},
            "selectionRange": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 6}}}
        with mock.patch.object(probe, "authenticated_egui_member", return_value=(Path("/tmp/target.rs"), b"target")), \
             mock.patch.object(probe, "authenticated_epaint_member") as epaint:
            authenticated = probe._target(target)
        epaint.assert_not_called()
        self.assertEqual(authenticated["resolution_kind"], "authenticated_egui_member_source_identity_only")
        self.assertFalse(authenticated["target_name_verified"])
        with mock.patch.object(probe, "authenticated_egui_member", return_value=None), \
             mock.patch.object(probe, "authenticated_epaint_member", return_value=(Path("/tmp/target.rs"), b"target")):
            authenticated = probe._target(target)
        with mock.patch.object(probe, "authenticated_egui_member", return_value=None), \
             mock.patch.object(probe, "authenticated_epaint_member", return_value=None):
            unresolved = probe._target(target)
        self.assertEqual(authenticated["resolution_kind"], "authenticated_epaint_member_source_identity_only")
        self.assertEqual(authenticated["target_package"], "epaint")
        self.assertFalse(authenticated["semantic_complete"])
        self.assertFalse(authenticated["target_name_verified"])
        self.assertEqual(unresolved["resolution_kind"], "unresolved_external_boundary")
        self.assertIsNone(unresolved["target_package"])

    def test_sorts_from_ranges_with_their_matching_utf16_excerpts(self):
        text = _text()
        root = _root(text, "on_event")
        first = probe._range(text, text.index("日本"), text.index("日本") + len("日本"))
        second = probe._range(text, text.index("self.on_key_press"),
                              text.index("self.on_key_press") + len("self.on_key_press"))
        target = {"name": "target", "uri": "file:///tmp/unknown.rs", "range": {
            "start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 6}},
            "selectionRange": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 6}}}
        with mock.patch.object(probe, "authenticated_epaint_member", return_value=None):
            outgoing = probe._outgoing(FakeClient([[{"to": target, "fromRanges": [second, first]}]]), root, text)
        call = outgoing["calls"][0]
        self.assertEqual(call["fromRanges"], [first, second])
        self.assertEqual([item["text"] for item in call["from_range_excerpts"]], ["日本", "self.on_key_press"])


if __name__ == "__main__":
    unittest.main()
