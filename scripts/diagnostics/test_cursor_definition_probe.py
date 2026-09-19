import importlib.util
import io
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
from unittest import mock


DIRECTORY = Path(__file__).parent
sys.path.insert(0, str(DIRECTORY))
SPEC = importlib.util.spec_from_file_location("cursor_definition_probe", DIRECTORY / "cursor_definition_probe.py")
probe = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(probe)
import egui_definition_probe as egui_probe


class FakeClient:
    def __init__(self, results):
        self.results = iter(results)
        self.sent = []

    def send(self, message):
        self.sent.append(message)

    def request(self, method, params):
        self.sent.append((method, params))
        return next(self.results)


def _texts():
    builder = "fn show() {}\n😀cursor_range.on_event(os, event, galley, id)\n"
    cursor = """impl CCursorRange {
pub fn on_key_press() {
    match key {
        Key::ArrowLeft
        | Key::End => {
                move_single_cursor(
                    os,
                );
        }
        Key::P
            if os == OperatingSystem::Mac && modifiers.ctrl && !modifiers.shift =>
            {
                move_single_cursor(
                    os,
                );
        }
    }
}

pub fn on_event() {
    match event {
        Event::Key {} => self.on_key_press(os, galley, modifiers, *key),
    }
}

fn move_single_cursor() {}
}
"""
    return builder, cursor


def _method_symbol(text, name, end):
    start = text.index(f"pub fn {name}(")
    return {"name": name, "range": probe._range(text, start, end),
            "selectionRange": probe._symbol_range(text, name)}


def _symbols(text):
    on_key_start = text.index("pub fn on_key_press(")
    on_event_start = text.index("pub fn on_event(")
    move_start = text.index("fn move_single_cursor(")
    owner = {"name": "impl CCursorRange", "range": probe._range(text, 0, len(text)),
             "selectionRange": probe._impl_range(text), "children": [
                 _method_symbol(text, "on_key_press", on_event_start),
                 _method_symbol(text, "on_event", move_start),
             ]}
    return [owner]


class CursorDefinitionTests(unittest.TestCase):
    def _fixture(self, root, builder, cursor):
        registry = "registry-id"
        source_root = root / ".cargo" / "registry" / "src" / registry
        builder_path = source_root / egui_probe.BUILDER_SOURCE_SUFFIX
        cursor_path = source_root / probe.CURSOR_RANGE_SOURCE_SUFFIX
        builder_path.parent.mkdir(parents=True, exist_ok=True)
        cursor_path.parent.mkdir(parents=True, exist_ok=True)
        builder_path.write_bytes(builder.encode("utf-8"))
        cursor_path.write_bytes(cursor.encode("utf-8"))
        archive = root / ".cargo" / "registry" / "cache" / registry / "egui-0.36.1.crate"
        archive.parent.mkdir(parents=True, exist_ok=True)
        with tarfile.open(archive, "w") as target:
            for member, text in ((egui_probe.BUILDER_MEMBER, builder), (probe.CURSOR_RANGE_MEMBER, cursor)):
                info = tarfile.TarInfo(member)
                data = text.encode("utf-8")
                info.size = len(data)
                target.addfile(info, io.BytesIO(data))
        return builder_path, cursor_path, egui_probe.hashlib.sha256(archive.read_bytes()).hexdigest()

    def test_follows_fixed_cursor_chain_with_utf16_prefix(self):
        builder, cursor = _texts()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            builder_path, cursor_path, digest = self._fixture(root, builder, cursor)
            client = FakeClient([
                [{"uri": cursor_path.as_uri(), "range": probe._symbol_range(cursor, "on_event")}],
                _symbols(cursor),
                [{"uri": cursor_path.as_uri(), "range": probe._symbol_range(cursor, "on_key_press")}],
                [{"uri": cursor_path.as_uri(), "range": probe._symbol_range(cursor, "move_single_cursor")}],
                [{"uri": cursor_path.as_uri(), "range": probe._symbol_range(cursor, "move_single_cursor")}],
            ])
            show = [{"uri": builder_path.as_uri(), "range": probe._symbol_range(builder, "show")}]
            with mock.patch.object(egui_probe.Path, "home", return_value=root), \
                 mock.patch.object(egui_probe, "ARCHIVE_SHA256", digest):
                edges = probe.follow_cursor_range(client, show)
        self.assertEqual([edge["name"] for edge in edges], [
            "cursor_range.on_event", "on_key_press", "move_single_cursor_navigation",
            "move_single_cursor_macos_control"])
        self.assertEqual(edges[0]["query_position"], {"line": 1, "character": 15})
        self.assertEqual(edges[1]["owner_range"], _symbols(cursor)[0]["children"][1]["range"])
        self.assertEqual(edges[2]["owner_range"], _symbols(cursor)[0]["children"][0]["range"])
        self.assertTrue(all(not edge["semantic_complete"] for edge in edges))

    def test_rejects_wrong_uri_range_and_multiple_results(self):
        builder, cursor = _texts()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            builder_path, cursor_path, digest = self._fixture(root, builder, cursor)
            show = [{"uri": builder_path.as_uri(), "range": probe._symbol_range(builder, "show")}]
            cases = [
                [{"uri": (root / "wrong.rs").as_uri(), "range": probe._symbol_range(cursor, "on_event")}],
                [{"uri": cursor_path.as_uri(), "range": {} }],
                [{"uri": cursor_path.as_uri(), "range": probe._symbol_range(cursor, "on_event")}] * 2,
            ]
            for result in cases:
                with self.subTest(result=result), mock.patch.object(egui_probe.Path, "home", return_value=root), \
                     mock.patch.object(egui_probe, "ARCHIVE_SHA256", digest), self.assertRaises(ValueError):
                    probe.follow_cursor_range(FakeClient([result]), show)

    def test_rejects_ambiguous_callsite_and_identifier_range(self):
        builder, cursor = _texts()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            builder_path, cursor_path, digest = self._fixture(root, builder, cursor)
            show = [{"uri": builder_path.as_uri(), "range": probe._symbol_range(builder, "show")}]
            ambiguous_builder_path, _, ambiguous_digest = self._fixture(
                root, builder + probe.BUILDER_CALL, cursor)
            with mock.patch.object(egui_probe.Path, "home", return_value=root), \
                 mock.patch.object(egui_probe, "ARCHIVE_SHA256", ambiguous_digest), self.assertRaisesRegex(ValueError, "ambiguous query anchor"):
                probe.follow_cursor_range(FakeClient([]), [{
                    "uri": ambiguous_builder_path.as_uri(), "range": probe._symbol_range(builder, "show")
                }])
            builder_path, cursor_path, digest = self._fixture(root, builder, cursor)
            show = [{"uri": builder_path.as_uri(), "range": probe._symbol_range(builder, "show")}]
            callsite = cursor.index("on_key_press", cursor.index("self."))
            client = FakeClient([
                [{"uri": cursor_path.as_uri(), "range": probe._symbol_range(cursor, "on_event")}],
                _symbols(cursor),
                [{"uri": cursor_path.as_uri(), "range": egui_probe._range(cursor, callsite, callsite + len("on_key_press"))}],
            ])
            with mock.patch.object(egui_probe.Path, "home", return_value=root), \
                 mock.patch.object(egui_probe, "ARCHIVE_SHA256", digest), self.assertRaisesRegex(ValueError, "exact authenticated symbol"):
                probe.follow_cursor_range(client, show)

    def test_rejects_query_anchor_moved_outside_its_owner(self):
        builder, cursor = _texts()
        moved = cursor.replace(
            "Event::Key {} => self.on_key_press(os, galley, modifiers, *key),",
            "Event::Key {} => unrelated(),") + "\npub fn unrelated() {\n    } => self.on_key_press(os, galley, modifiers, *key),\n}\n"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            builder_path, cursor_path, digest = self._fixture(root, builder, moved)
            show = [{"uri": builder_path.as_uri(), "range": probe._symbol_range(builder, "show")}]
            client = FakeClient([[{
                "uri": cursor_path.as_uri(), "range": probe._symbol_range(moved, "on_event")
            }], _symbols(moved)])
            with mock.patch.object(egui_probe.Path, "home", return_value=root), \
                 mock.patch.object(egui_probe, "ARCHIVE_SHA256", digest), \
                 self.assertRaisesRegex(ValueError, "outside structured owner: on_key_press"):
                probe.follow_cursor_range(client, show)

    def test_rejects_undefined_multiple_and_malformed_document_symbol_owners(self):
        builder, cursor = _texts()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            builder_path, cursor_path, digest = self._fixture(root, builder, cursor)
            show = [{"uri": builder_path.as_uri(), "range": probe._symbol_range(builder, "show")}]
            malformed_selection = _symbols(cursor)
            malformed_selection[0]["selectionRange"] = {}
            inverted_range = _symbols(cursor)
            inverted_range[0]["range"] = {"start": {"line": 1, "character": 0},
                                            "end": {"line": 0, "character": 0}}
            bool_range = _symbols(cursor)
            bool_range[0]["range"]["start"]["line"] = True
            method_outside_owner = _symbols(cursor)
            method_outside_owner[0]["range"] = probe._range(
                cursor, 0, cursor.index("\n"))
            missing_children = _symbols(cursor)
            missing_children[0].pop("children")
            unknown_method = _symbols(cursor)
            unknown_method[0]["children"] = [{"name": "unknown", "range": probe._range(cursor, 0, len(cursor)),
                                                "selectionRange": probe._range(cursor, 0, 1)}]
            cases = [
                ([], "exactly one CCursorRange impl owner"),
                (_symbols(cursor) * 2, "exactly one CCursorRange impl owner"),
                (malformed_selection, "selection range"),
                (inverted_range, "range is malformed"),
                (bool_range, "range is malformed"),
                (method_outside_owner, "method range is outside"),
                (missing_children, "missing method symbols"),
                (unknown_method, "exactly one on_event symbol"),
            ]
            for symbols, message in cases:
                client = FakeClient([
                    [{"uri": cursor_path.as_uri(), "range": probe._symbol_range(cursor, "on_event")}], symbols
                ])
                with self.subTest(symbols=symbols), \
                     mock.patch.object(egui_probe.Path, "home", return_value=root), \
                     mock.patch.object(egui_probe, "ARCHIVE_SHA256", digest), \
                     self.assertRaisesRegex(ValueError, message):
                    probe.follow_cursor_range(client, show)

    def test_real_archive_authenticates_cursor_range_and_fixed_symbols(self):
        sources = list((Path.home() / ".cargo" / "registry" / "src").glob(
            "*/egui-0.36.1/src/text_selection/cursor_range.rs"))
        if not sources:
            self.skipTest("egui 0.36.1 is not cached")
        source, data = egui_probe._authenticated_source(
            sources[0].as_uri(), probe.CURSOR_RANGE_MEMBER, probe.CURSOR_RANGE_SOURCE_SUFFIX, "cursor_range")
        text = data.decode("utf-8")
        self.assertEqual(source, sources[0])
        self.assertEqual(len([line for line in text.splitlines() if "move_single_cursor(" in line]), 3)
        self.assertEqual(text.count(probe.ON_KEY_PRESS_CALL), 1)
        self.assertEqual(sum(text.count(anchor) for _, anchor in probe.MOVE_SINGLE_CURSOR_CALLS), 2)


if __name__ == "__main__":
    unittest.main()
