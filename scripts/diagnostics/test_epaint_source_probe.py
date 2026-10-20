import hashlib
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
SPEC = importlib.util.spec_from_file_location("epaint_source_probe", DIRECTORY / "epaint_source_probe.py")
probe = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(probe)


def _archive(entries):
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:gz") as source:
        for name, data in entries:
            entry = tarfile.TarInfo(name)
            entry.size = len(data)
            source.addfile(entry, io.BytesIO(data))
    return output.getvalue()


class EpaintSourceProbeTests(unittest.TestCase):
    def _fixture(self, entries=None, source=b"pub struct LayoutJob;\n"):
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name) / "src"
        target = root / "registry-id" / "epaint-0.36.1" / "src" / "text" / "text_layout_types.rs"
        target.parent.mkdir(parents=True)
        target.write_bytes(source)
        member = "epaint-0.36.1/src/text/text_layout_types.rs"
        archive_bytes = _archive(entries if entries is not None else [(member, source)])
        archive = root.parent / "cache" / "registry-id" / "epaint-0.36.1.crate"
        archive.parent.mkdir(parents=True)
        archive.write_bytes(archive_bytes)
        return temporary, root, target, archive_bytes

    def _authenticate(self, root, target, archive_bytes):
        with mock.patch.object(probe, "CARGO_REGISTRY_SRC", root), \
             mock.patch.object(probe, "ARCHIVE_SHA256", hashlib.sha256(archive_bytes).hexdigest()):
            return probe.authenticated_epaint_member(target.as_uri())

    def test_authenticates_exact_text_layout_types_member(self):
        temporary, root, target, archive_bytes = self._fixture()
        with temporary:
            path, data = self._authenticate(root, target, archive_bytes)
        self.assertEqual(path, target.resolve())
        self.assertEqual(data, b"pub struct LayoutJob;\n")

    def test_rejects_bad_archive_sha_duplicate_missing_member_and_modified_source(self):
        member = "epaint-0.36.1/src/text/text_layout_types.rs"
        cases = [([], "missing or duplicated"), ([(member, b"a"), (member, b"a")], "missing or duplicated"),
                 ([(member, b"archive")], "differs from the archive")]
        for entries, message in cases:
            with self.subTest(message=message):
                temporary, root, target, archive_bytes = self._fixture(entries)
                with temporary, self.assertRaisesRegex(ValueError, message):
                    self._authenticate(root, target, archive_bytes)
        temporary, root, target, archive_bytes = self._fixture()
        with temporary, mock.patch.object(probe, "CARGO_REGISTRY_SRC", root), \
             self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
            probe.authenticated_epaint_member(target.as_uri())

    def test_rejects_non_epaint_or_non_rust_paths_traversal_and_remote_uris(self):
        temporary, root, target, archive_bytes = self._fixture()
        cases = [root / "registry-id" / "egui-0.36.1" / "src" / "a.rs",
                 root / "registry-id" / "epaint-0.36.2" / "src" / "a.rs",
                 root / "registry-id" / "epaint-0.36.1" / "src" / "a.txt"]
        with temporary, mock.patch.object(probe, "CARGO_REGISTRY_SRC", root):
            for path in cases:
                with self.subTest(path=path):
                    self.assertIsNone(probe.authenticated_epaint_member(path.as_uri()))
            self.assertIsNone(probe.authenticated_epaint_member("https://example.test/epaint.rs"))
            self.assertIsNone(probe.authenticated_epaint_member(
                target.parent.as_uri() + "/../text_layout_types.rs"))


if __name__ == "__main__":
    unittest.main()
