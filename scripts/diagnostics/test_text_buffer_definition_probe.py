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
MODULE_PATH = DIRECTORY / "text_buffer_definition_probe.py"
SPEC = importlib.util.spec_from_file_location("text_buffer_definition_probe", MODULE_PATH)
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


def _source_texts():
    builder = "fn show() {}\n😀⭐️text.delete_previous_char(cursor)\n"
    text_buffer = """pub trait TextBuffer {
    fn delete_char_range(&mut self, char_range: Range<CharIndex>);

    fn delete_selected_ccursor_range(&mut self, [min, max]: [CCursor; 2]) -> CCursor {
        self.delete_char_range(min.index..max.index);
        min
    }

    fn delete_previous_char(&mut self, ccursor: CCursor) -> CCursor {
        self.delete_selected_ccursor_range([min_ccursor, max_ccursor])
    }

    fn delete_previous_word(&mut self, ccursor: CCursor) -> CCursor {
        self.delete_selected_ccursor_range([min_ccursor, max_ccursor])
    }
}
"""
    return builder, text_buffer


def _show_result(builder_path, builder):
    start = builder.index("fn show") + len("fn ")
    return [{"uri": builder_path.as_uri(), "range": probe._range(builder, start, start + len("show"))}]


class TextBufferDefinitionTests(unittest.TestCase):
    def _authenticated_fixture(self, root, builder, text_buffer):
        registry = "registry-id"
        source_root = root / ".cargo" / "registry" / "src" / registry
        builder_path = source_root / egui_probe.BUILDER_SOURCE_SUFFIX
        text_buffer_path = source_root / probe.TEXT_BUFFER_SOURCE_SUFFIX
        builder_path.parent.mkdir(parents=True)
        builder_path.write_text(builder, encoding="utf-8")
        text_buffer_path.write_text(text_buffer, encoding="utf-8")
        archive = root / ".cargo" / "registry" / "cache" / registry / "egui-0.36.1.crate"
        archive.parent.mkdir(parents=True)
        with tarfile.open(archive, "w") as target:
            for member, data in ((egui_probe.BUILDER_MEMBER, builder),
                                 (probe.TEXT_BUFFER_MEMBER, text_buffer)):
                info = tarfile.TarInfo(member)
                encoded = data.encode("utf-8")
                info.size = len(encoded)
                target.addfile(info, io.BytesIO(encoded))
        digest = egui_probe.hashlib.sha256(archive.read_bytes()).hexdigest()
        return builder_path, text_buffer_path, digest

    def test_follows_three_edges_with_utf16_prefix_and_dynamic_dispatch(self):
        builder, text_buffer = _source_texts()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            builder_path, text_buffer_path, digest = self._authenticated_fixture(root, builder, text_buffer)
            previous = probe._symbol_range(text_buffer, "delete_previous_char")
            selected = probe._symbol_range(text_buffer, "delete_selected_ccursor_range")
            abstract = probe._symbol_range(text_buffer, "delete_char_range")
            client = FakeClient([
                [{"uri": text_buffer_path.as_uri(), "range": previous}],
                [{"uri": text_buffer_path.as_uri(), "range": selected}],
                [{"uri": text_buffer_path.as_uri(), "range": abstract}],
            ])
            with mock.patch.object(egui_probe.Path, "home", return_value=root), \
                 mock.patch.object(egui_probe, "ARCHIVE_SHA256", digest):
                edges = probe.follow_text_buffer(client, _show_result(builder_path, builder))
        self.assertEqual([edge["name"] for edge in edges], [
            "delete_previous_char", "delete_selected_ccursor_range", "delete_char_range"])
        self.assertEqual(edges[0]["query_position"], {"line": 1, "character": 9})
        owner_call = text_buffer.index("self.delete_selected_ccursor_range([min_ccursor, max_ccursor])")
        self.assertEqual(edges[1]["query_position"], probe._position(text_buffer, owner_call + len("self.")))
        self.assertEqual(edges[-1]["resolution_kind"], "unresolved_dynamic_dispatch")
        self.assertTrue(all(not edge["semantic_complete"] for edge in edges))
        self.assertEqual(sum(message["method"] == "textDocument/didOpen"
                             for message in client.sent if isinstance(message, dict)), 1)

    def test_rejects_callsite_instead_of_default_method_definition(self):
        builder, text_buffer = _source_texts()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            builder_path, text_buffer_path, digest = self._authenticated_fixture(root, builder, text_buffer)
            previous = probe._symbol_range(text_buffer, "delete_previous_char")
            callsite = text_buffer.index("delete_selected_ccursor_range([")
            client = FakeClient([
                [{"uri": text_buffer_path.as_uri(), "range": previous}],
                [{"uri": text_buffer_path.as_uri(), "range": probe._range(
                    text_buffer, callsite, callsite + len("delete_selected_ccursor_range"))}],
            ])
            with mock.patch.object(egui_probe.Path, "home", return_value=root), \
                 mock.patch.object(egui_probe, "ARCHIVE_SHA256", digest), \
                 self.assertRaisesRegex(ValueError, "exact authenticated symbol"):
                probe.follow_text_buffer(client, _show_result(builder_path, builder))

    def test_rejects_dynamic_dispatch_callsite_results(self):
        builder, text_buffer = _source_texts()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            builder_path, text_buffer_path, digest = self._authenticated_fixture(root, builder, text_buffer)
            previous = probe._symbol_range(text_buffer, "delete_previous_char")
            selected = probe._symbol_range(text_buffer, "delete_selected_ccursor_range")
            callsite = text_buffer.index("delete_char_range(min")
            client = FakeClient([
                [{"uri": text_buffer_path.as_uri(), "range": previous}],
                [{"uri": text_buffer_path.as_uri(), "range": selected}],
                [{"uri": text_buffer_path.as_uri(), "range": probe._range(
                    text_buffer, callsite, callsite + len("delete_char_range"))}],
            ])
            with mock.patch.object(egui_probe.Path, "home", return_value=root), \
                 mock.patch.object(egui_probe, "ARCHIVE_SHA256", digest), \
                 self.assertRaisesRegex(ValueError, "authenticated definition identifier"):
                probe.follow_text_buffer(client, _show_result(builder_path, builder))

    def test_rejects_duplicate_builder_anchor_before_lsp_request(self):
        builder, text_buffer = _source_texts()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            builder_path, _, digest = self._authenticated_fixture(
                root, builder + "text.delete_previous_char(cursor)\n", text_buffer)
            with mock.patch.object(egui_probe.Path, "home", return_value=root), \
                 mock.patch.object(egui_probe, "ARCHIVE_SHA256", digest), \
                 self.assertRaisesRegex(ValueError, "ambiguous query anchor: delete_previous_char"):
                probe.follow_text_buffer(FakeClient([]), _show_result(builder_path, builder))

    def test_rejects_non_symbol_show_range_after_builder_authentication(self):
        builder, text_buffer = _source_texts()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            builder_path, _, digest = self._authenticated_fixture(root, builder, text_buffer)
            with mock.patch.object(egui_probe.Path, "home", return_value=root), \
                 mock.patch.object(egui_probe, "ARCHIVE_SHA256", digest), \
                 self.assertRaisesRegex(ValueError, "exact authenticated show symbol"):
                probe.follow_text_buffer(FakeClient([]), [{
                    "uri": builder_path.as_uri(), "range": probe._range(builder, 0, 2)}])

    def test_real_cached_text_buffer_has_expected_definition_identifiers(self):
        source_root = Path.home() / ".cargo" / "registry" / "src"
        sources = list(source_root.glob("*/egui-0.36.1/src/widgets/text_edit/text_buffer.rs"))
        if not sources:
            self.skipTest("egui 0.36.1 is not cached")
        source, data = egui_probe._authenticated_source(
            sources[0].as_uri(), probe.TEXT_BUFFER_MEMBER, probe.TEXT_BUFFER_SOURCE_SUFFIX, "text_buffer")
        text = data.decode("utf-8")
        self.assertEqual(source, sources[0])
        self.assertEqual(egui_probe.hashlib.sha256(data).hexdigest(),
                         "6ac11dc0237bf7250bfae23c8139c56ae352ecb92b9ecfa699bc5b73203d0989")
        self.assertEqual(len(probe._symbol_ranges(text, "delete_previous_char")), 1)
        self.assertEqual(len(probe._symbol_ranges(text, "delete_selected_ccursor_range")), 1)
        self.assertEqual(len(probe._symbol_ranges(text, "delete_char_range")), 4)


if __name__ == "__main__":
    unittest.main()
