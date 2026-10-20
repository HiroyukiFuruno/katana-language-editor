import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from dependency import dependency_changed, dependency_entries
from cli import parse_push_lines
from cli import require_clean_worktree
from git_io import blob, changed_manifests, origin_identity, parse_git_trailers, run_git
from policy import PolicyError, parse_issue_json, require_open_issue, validate_ref


def git(root: Path, *args: str, input_text: str | None = None) -> str:
    result = subprocess.run(["git", *args], cwd=root, input=input_text, text=True, capture_output=True, check=True)
    return result.stdout


class GitFixtureTests(unittest.TestCase):
    def test_git_trailer_parser_and_blob_error_classification(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            git(root, "init", "-q")
            git(root, "config", "user.email", "test@example.com")
            git(root, "config", "user.name", "Test")
            (root / "Cargo.toml").write_text("[dependencies]\nkatana-ui-core = \"0.4\"\n", encoding="utf-8")
            git(root, "add", ".")
            git(root, "commit", "-qm", "Subject\n\nIssue: #9")
            commit = git(root, "rev-parse", "HEAD").strip()
            trailers = parse_git_trailers(root, git(root, "show", "-s", "--format=%B", commit))
            self.assertEqual(trailers.issue, 9)
            self.assertIsNone(trailers.downstream_issue)
            self.assertIsNone(blob(root, commit, "missing.toml"))
            with self.assertRaises(PolicyError):
                blob(root, "not-a-revision", "Cargo.toml")

    def test_workspace_alias_resolves_root_package_and_ignores_metadata(self):
        member = b'''[package]\nname = "member"\n[dependencies]\nui = { workspace = true }\n[package.metadata.dependencies]\nfake = { package = "katana-fake", version = "1" }\n'''
        self.assertEqual(len(dependency_entries(member, {"ui": {"package": "katana-ui-core", "version": "0.4"}})), 1)
        changed = member.replace(b"0.4", b"0.5")
        self.assertTrue(dependency_changed(member, changed, before_workspace={"ui": {"package": "katana-ui-core", "version": "0.4"}}, after_workspace={"ui": {"package": "katana-ui-core", "version": "0.5"}}))
        self.assertFalse(dependency_changed(member, member, before_workspace={"ui": {"package": "katana-ui-core", "version": "0.4"}}, after_workspace={"ui": {"package": "katana-ui-core", "version": "0.4"}}))

    def test_dependency_overrides_and_legitimate_target_tables(self):
        root = b'''[workspace.dependencies]\nui = { package = "katana-ui-core", version = "0.4", default-features = false }\n'''
        before = b'''[dependencies]\nui = { workspace = true, features = ["x"], optional = true }\n[dev-dependencies]\nkatana-dev = "1"\n[target.'cfg(unix)'.build-dependencies]\nkatana-build = "1"\n[target.'cfg(unix)'.metadata.dependencies]\nkatana-ignored = "1"\n'''
        after = before.replace(b'["x"]', b'["y"]').replace(b'katana-dev = "1"', b'katana-dev = "2"')
        self.assertTrue(dependency_changed(before, after, before_workspace=root, after_workspace=root))
        self.assertEqual(len(dependency_entries(before, {"ui": {"package": "katana-ui-core", "version": "0.4", "default-features": False}})), 3)

    def test_invalid_workspace_inheritance_fails_closed(self):
        with self.assertRaises(PolicyError):
            dependency_entries(b'[dependencies]\nui = { workspace = true }\n', {})
        with self.assertRaises(PolicyError):
            dependency_entries(b'[workspace.dependencies]\nui = { workspace = true }\n', {"ui": {"workspace": True}})

    def test_manifest_history_detects_workspace_alias_change(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            git(root, "init", "-q")
            git(root, "config", "user.email", "test@example.com")
            git(root, "config", "user.name", "Test")
            (root / "Cargo.toml").write_text('[workspace]\nmembers = ["member"]\n[workspace.dependencies]\nui = { package = "katana-ui-core", version = "0.4" }\n', encoding="utf-8")
            (root / "member").mkdir()
            (root / "member/Cargo.toml").write_text('[package]\nname = "member"\n[dependencies]\nui = { workspace = true }\n', encoding="utf-8")
            git(root, "add", ".")
            git(root, "commit", "-qm", "initial")
            parent = git(root, "rev-parse", "HEAD").strip()
            (root / "Cargo.toml").write_text((root / "Cargo.toml").read_text(encoding="utf-8").replace("0.4", "0.5"), encoding="utf-8")
            git(root, "commit", "-qam", "update")
            commit = git(root, "rev-parse", "HEAD").strip()
            self.assertTrue(changed_manifests(root, parent, commit))

    def test_origin_requires_anchored_github_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            git(root, "init", "-q")
            git(root, "remote", "add", "origin", "https://github.com/acme/repo.git")
            self.assertEqual(origin_identity(root), "acme/repo")
            git(root, "remote", "set-url", "origin", "https://evilgithub.com/acme/repo.git")
            with self.assertRaises(PolicyError):
                origin_identity(root)

    def test_clean_worktree_and_stash_are_required_by_push_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            git(root, "init", "-q")
            git(root, "config", "user.email", "test@example.com")
            git(root, "config", "user.name", "Test")
            (root / "tracked").write_text("clean\n", encoding="utf-8")
            git(root, "add", ".")
            git(root, "commit", "-qm", "initial")
            require_clean_worktree(root)
            (root / "untracked").write_text("dirty\n", encoding="utf-8")
            with self.assertRaises(PolicyError):
                require_clean_worktree(root)
            (root / "untracked").unlink()
            (root / "tracked").write_text("stashed\n", encoding="utf-8")
            git(root, "stash", "push", "-qm", "fixture")
            with self.assertRaises(PolicyError):
                require_clean_worktree(root)
            with self.assertRaises(PolicyError):
                run_git(root, "check-ref-format", "refs/heads/release/")


class PolicyFixtureTests(unittest.TestCase):
    def test_push_input_requires_records_and_normal_deletion_marker(self):
        with self.assertRaises(PolicyError):
            parse_push_lines("")
        parsed = parse_push_lines(f"(delete) {'0' * 40} refs/heads/release/old {'a' * 40}\n")
        self.assertEqual(parsed[0][0], "(delete)")
        with self.assertRaises(PolicyError):
            parse_push_lines(f"(delete) {'a' * 40} refs/heads/release/old {'b' * 40}\n")

    def test_issue_json_requires_exact_issue_shape(self):
        raw = '{"number":9,"html_url":"https://github.com/acme/repo/issues/9","state":"open","body":""}'
        self.assertEqual(parse_issue_json(raw, expected_number=9, expected_repo="acme/repo"), ("open", ""))
        for bad in ("not json", '{"number":14,"html_url":"https://github.com/acme/repo/issues/14","state":"open","body":""}', '{"number":true,"html_url":"https://github.com/acme/repo/issues/1","state":"open","body":""}', '{"number":9,"html_url":"https://github.com/other/repo/issues/9","state":"open","body":""}', '{"number":9,"html_url":"https://github.com/acme/repo/issues/9","state":"open","body":null}', '{"number":9,"html_url":"https://github.com/acme/repo/issues/9","state":"open","body":"","pull_request":{}}'):
            with self.subTest(bad=bad), self.assertRaises(PolicyError):
                require_open_issue(bad, expected_number=9, expected_repo="acme/repo")

    def test_metadata_requires_nonempty_same_line_values(self):
        fields = ["Upstream-Release", "Dependency-Update", "Validation"]
        for empty in fields:
            body = "\n".join(f"{name}: {'value' if name != empty else ''}" for name in fields)
            raw = json.dumps({"number": 9, "html_url": "https://github.com/acme/repo/issues/9", "state": "open", "body": body})
            with self.subTest(empty=empty), self.assertRaises(PolicyError):
                require_open_issue(raw, expected_number=9, expected_repo="acme/repo", metadata=True)

    def test_refs_allow_nondefault_deletion_but_require_release_updates(self):
        oid = "a" * 40
        zero = "0" * 40
        validate_ref("refs/heads/release/v0.1.0", "main", oid, zero)
        validate_ref("refs/heads/issue/9", "main", oid, zero)
        for ref, old, new in (("refs/heads/main", oid, oid), ("refs/heads/topic", oid, oid), ("refs/tags/v1", oid, zero), ("refs/heads/release/x", "a" * 40, "b" * 64), ("refs/heads/release/x", oid, "0" * 64)):
            with self.subTest(ref=ref), self.assertRaises(PolicyError):
                validate_ref(ref, "main", old, new)
        with self.assertRaises(PolicyError):
            validate_ref("refs/heads/release/x..y", "main", oid, oid)


if __name__ == "__main__":
    unittest.main()
