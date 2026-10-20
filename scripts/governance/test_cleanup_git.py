import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from cleanup_io import ancestor, bound_branches, local_branches, remote_branches, stash_present, worktree_dirty


def git(root: Path, *args: str) -> str:
    return subprocess.run(["git", *args], cwd=root, text=True, capture_output=True, check=True).stdout


class GitCleanupFactsTests(unittest.TestCase):
    def test_local_branch_records_keep_real_shas(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            git(root, "init", "-q")
            git(root, "config", "user.email", "test@example.com")
            git(root, "config", "user.name", "Test")
            (root / "tracked").write_text("one\n", encoding="utf-8")
            git(root, "add", ".")
            git(root, "commit", "-qm", "initial")
            for name in ("release/one", "release/two", "release/three"):
                git(root, "branch", name)
            branches = dict(local_branches(root))
            self.assertGreaterEqual(len(branches), 4)
            self.assertTrue(all(len(sha) == 40 for sha in branches.values()))

    def test_ancestry_and_dirty_stash_facts_use_real_repository(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            git(root, "init", "-q")
            git(root, "config", "user.email", "test@example.com")
            git(root, "config", "user.name", "Test")
            (root / "tracked").write_text("one\n", encoding="utf-8")
            git(root, "add", ".")
            git(root, "commit", "-qm", "initial")
            first = git(root, "rev-parse", "HEAD").strip()
            (root / "tracked").write_text("two\n", encoding="utf-8")
            git(root, "commit", "-qam", "second")
            second = git(root, "rev-parse", "HEAD").strip()
            self.assertTrue(ancestor(root, first, second))
            self.assertFalse(ancestor(root, second, first))
            (root / "untracked").write_text("dirty\n", encoding="utf-8")
            self.assertTrue(worktree_dirty(root))
            (root / "untracked").unlink()
            (root / "tracked").write_text("stashed\n", encoding="utf-8")
            git(root, "stash", "push", "-qm", "fixture")
            self.assertTrue(stash_present(root))

    def test_worktree_records_and_bare_remote_without_creating_worktrees(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            bare = root / "remote.git"
            git(root, "init", "--bare", "-q", str(bare))
            git(root, "init", "-q")
            git(root, "config", "user.email", "test@example.com")
            git(root, "config", "user.name", "Test")
            (root / "tracked").write_text("one\n", encoding="utf-8")
            git(root, "add", ".")
            git(root, "commit", "-qm", "initial")
            git(root, "remote", "add", "origin", str(bare))
            git(root, "push", "-q", "-u", "origin", "HEAD:release/fixture")
            git(root, "fetch", "-q", "origin")
            self.assertEqual(remote_branches(root)[0][0], "release/fixture")
            current = git(root, "symbolic-ref", "--short", "HEAD").strip()
            self.assertIn(current, bound_branches(root))

    def test_real_remote_has_three_distinct_release_tips(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            bare = root / "remote.git"
            git(root, "init", "--bare", "-q", str(bare))
            git(root, "init", "-q")
            git(root, "config", "user.email", "test@example.com")
            git(root, "config", "user.name", "Test")
            git(root, "remote", "add", "origin", str(bare))
            (root / "tracked").write_text("base\n", encoding="utf-8")
            git(root, "add", ".")
            git(root, "commit", "-qm", "base")
            for index, name in enumerate(("one", "two", "three"), 1):
                git(root, "switch", "-c", f"release/{name}")
                (root / "tracked").write_text(f"{index}\n", encoding="utf-8")
                git(root, "commit", "-qam", name)
                git(root, "push", "-q", "origin", f"release/{name}")
                git(root, "switch", "-q", "-C", "main", "HEAD~1")
            git(root, "fetch", "-q", "origin")
            refs = dict(remote_branches(root))
            self.assertEqual(set(refs), {"release/one", "release/two", "release/three"})
            self.assertEqual(len(set(refs.values())), 3)


if __name__ == "__main__":
    unittest.main()
