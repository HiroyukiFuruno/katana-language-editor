import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest import mock


MODULE_PATH = Path(__file__).with_name("egui_definition_probe.py")
SPEC = importlib.util.spec_from_file_location("egui_definition_probe", MODULE_PATH)
probe = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(probe)


class FakeClient:
    def __init__(self, results):
        self.results = iter(results)
        self.sent = []

    def send(self, message):
        self.sent.append(message)

    def request(self, method, params):
        self.sent.append((method, params))
        return next(self.results)


class ValidationTests(unittest.TestCase):
    def test_windows_file_uri_keeps_the_drive_absolute(self):
        with mock.patch.object(probe.os, "name", "nt"):
            self.assertEqual(
                probe._local_file_uri_path("/C:/Users/runner/.cargo/registry/src/file.rs"),
                "C:/Users/runner/.cargo/registry/src/file.rs",
            )

    def test_location_rejects_null_multi_and_wrong_uri(self):
        for result in (None, [], [{"uri": "a", "range": {}}] * 2):
            with self.subTest(result=result), self.assertRaises(ValueError):
                probe._location(result)
        with self.assertRaises(ValueError):
            probe._path_from_uri("https://example.test/builder.rs")

    def test_archive_rejects_duplicate_member(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "egui.crate"
            with tarfile.open(archive, "w") as target:
                for _ in range(2):
                    info = tarfile.TarInfo(probe.MEMBER)
                    info.size = 1
                    target.addfile(info, io.BytesIO(b"x"))
            with mock.patch.object(probe.Path, "home", return_value=Path(directory).resolve()):
                source = Path(directory) / ".cargo" / "registry" / "src" / "registry-id" / probe.SOURCE_SUFFIX
                source.parent.mkdir(parents=True)
                source.write_bytes(b"x")
                cache = Path(directory) / ".cargo" / "registry" / "cache" / "registry-id"
                cache.mkdir(parents=True)
                archive.replace(cache / "egui-0.36.1.crate")
                digest = probe.hashlib.sha256((cache / "egui-0.36.1.crate").read_bytes()).hexdigest()
                with mock.patch.object(probe, "ARCHIVE_SHA256", digest), \
                     self.assertRaisesRegex(ValueError, "member is missing or duplicated"):
                    probe._authenticated_builder(source.as_uri())

    def test_modified_source_is_rejected_after_archive_authentication(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            source = root / ".cargo" / "registry" / "src" / "registry-id" / probe.SOURCE_SUFFIX
            source.parent.mkdir(parents=True)
            source.write_bytes(b"modified")
            archive = root / ".cargo" / "registry" / "cache" / "registry-id" / "egui-0.36.1.crate"
            archive.parent.mkdir(parents=True)
            with tarfile.open(archive, "w") as target:
                info = tarfile.TarInfo(probe.MEMBER)
                info.size = len(b"original")
                target.addfile(info, io.BytesIO(b"original"))
            digest = probe.hashlib.sha256(archive.read_bytes()).hexdigest()
            with mock.patch.object(probe.Path, "home", return_value=root), \
                 mock.patch.object(probe, "ARCHIVE_SHA256", digest), \
                 self.assertRaisesRegex(ValueError, "source differs from the archive"):
                probe._authenticated_builder(source.as_uri())

    def test_real_cached_archive_verification(self):
        source_root = Path.home() / ".cargo" / "registry" / "src"
        sources = list(source_root.glob("*/egui-0.36.1/src/widgets/text_edit/builder.rs"))
        if not sources:
            self.skipTest("egui 0.36.1 is not cached")
        source, data = probe._authenticated_builder(sources[0].as_uri())
        self.assertEqual(source, sources[0])
        self.assertEqual(probe.hashlib.sha256(data).hexdigest(),
                         probe.hashlib.sha256(sources[0].read_bytes()).hexdigest())

class AnchorTests(unittest.TestCase):
    def test_anchor_and_utf16_position_are_bounded(self):
        text = "x\nlet (changed, new_cursor_range) = events(\n"
        offset = text.index("new_cursor_range) = events(") + len("new_cursor_range) = ")
        self.assertEqual(probe._position(text, offset), {"line": 1, "character": 34})
        self.assertEqual(text.count("new_cursor_range) = events("), 1)

    def test_follow_edges_uses_declaration_not_callsite(self):
        text = ("fn show(&self) {}\n"
                "let (changed, new_cursor_range) = events(\n"
                "} => check_for_mutating_key_press(\n"
                "fn events() {}\n"
                "fn check_for_mutating_key_press() {}\n")
        uri = Path("/tmp/egui-builder.rs").as_uri()
        show_start = text.index("fn show") + 3
        events_start = text.index("fn events") + 3
        check_start = text.index("fn check_for_mutating_key_press") + 3
        good = FakeClient([
            [{"uri": uri, "range": probe._range(text, events_start, events_start + 6)}],
            [{"uri": uri, "range": probe._range(
                text, check_start, check_start + len("check_for_mutating_key_press"))}],
        ])
        with mock.patch.object(probe, "_authenticated_builder",
                               return_value=(Path("/tmp/egui-builder.rs"), text.encode())):
            edges = probe.follow_edges(
                good, [{"uri": uri, "range": probe._range(text, show_start, show_start + 4)}])
        self.assertEqual([edge["name"] for edge in edges],
                         ["events", "check_for_mutating_key_press"])
        self.assertNotEqual(edges[0]["position"], probe._position(text, events_start))

        callsite = FakeClient([
            [{"uri": uri, "range": probe._range(text, text.index("events", text.index("=") ),
                                                   text.index("events", text.index("=") ) + 6)}],
        ])
        with mock.patch.object(probe, "_authenticated_builder",
                               return_value=(Path("/tmp/egui-builder.rs"), text.encode())), \
             self.assertRaises(ValueError):
            probe.follow_edges(callsite,
                               [{"uri": uri, "range": probe._range(text, show_start, show_start + 4)}])

    def test_follow_edges_rejects_wrong_uri_range_and_duplicate_anchor(self):
        text = ("fn show(&self) {}\n"
                "let (changed, new_cursor_range) = events(\n"
                "} => check_for_mutating_key_press(\n"
                "fn events() {}\n"
                "fn check_for_mutating_key_press() {}\n")
        path = Path("/tmp/egui-builder.rs")
        uri = path.as_uri()
        show_start = text.index("fn show") + 3
        show_range = probe._range(text, show_start, show_start + 4)
        good_show = [{"uri": uri, "range": show_range}]
        with mock.patch.object(probe, "_authenticated_builder",
                               return_value=(path, text.encode())):
            with self.assertRaises(ValueError):
                probe.follow_edges(FakeClient([]), [{"uri": "file:///wrong", "range": show_range}])
            with self.assertRaises(ValueError):
                probe.follow_edges(FakeClient([]), [{"uri": uri, "range": {}}])

        duplicate = text.replace("fn events()", "fn events()\nfn events()")
        with mock.patch.object(probe, "_authenticated_builder",
                               return_value=(path, duplicate.encode())), \
             self.assertRaises(ValueError):
            probe.follow_edges(FakeClient([[{"uri": uri, "range": {}}]]), good_show)


if __name__ == "__main__":
    unittest.main()
