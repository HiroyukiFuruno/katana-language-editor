import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GOVERNANCE_FILES = (
    "cli.py",
    "dependency.py",
    "git_io.py",
    "policy.py",
    "validate-commit-message.sh",
    "validate-push.sh",
)
HOOK_FILES = (
    ".lefthook/commit-msg/issue-trailer.sh",
    ".lefthook/pre-push/check-strict.sh",
    ".lefthook/pre-push/issue-and-dependency.sh",
)


def run(root: Path, *args: str, input_text: str | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=root,
        input=input_text,
        text=True,
        capture_output=True,
        check=False,
        env={**os.environ, "GIT_TERMINAL_PROMPT": "0", "PYTHONDONTWRITEBYTECODE": "1"},
    )


class HookWiringIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.lefthook = shutil.which("lefthook")
        if cls.lefthook is None:
            raise RuntimeError("Lefthook is required for hook wiring integration tests")

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "repo"
        self.root.mkdir()
        self.bare = Path(self.temp.name) / "origin.git"
        self.assert_success(run(self.root, "git", "init", "-q"))
        self.assert_success(run(self.root, "git", "config", "user.name", "Hook Test"))
        self.assert_success(run(self.root, "git", "config", "user.email", "hook-test@example.invalid"))
        self.assert_success(run(self.root, "git", "init", "--bare", "-q", str(self.bare)))
        self._copy_production_hooks()
        self.assert_success(run(self.root, "git", "add", "."))
        self.assert_success(run(self.root, "git", "commit", "-m", "Install production hooks\n\nIssue: #9"))
        self.assert_success(run(self.root, "git", "remote", "add", "origin", str(self.bare)))
        self.assert_success(run(self.root, "git", "push", "-q", "-u", "origin", "HEAD:refs/heads/main"))

        self.assert_success(run(self.root, self.lefthook, "install"))
        status = run(self.root, "git", "status", "--porcelain", "--untracked-files=all")
        self.assert_success(status)
        self.assertEqual(status.stdout, "", f"fixture must be clean before tests: {status.stdout}")

    def _copy_production_hooks(self):
        for relative in ("lefthook.yml", *HOOK_FILES):
            destination = self.root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, destination)
        target = self.root / "scripts/governance"
        target.mkdir(parents=True)
        for name in GOVERNANCE_FILES:
            shutil.copy2(ROOT / "scripts/governance" / name, target / name)

    def assert_success(self, result: subprocess.CompletedProcess[str]):
        self.assertEqual(result.returncode, 0, f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}")

    def test_push_runs_dirty_guard_for_no_file_updates_and_stops_strict_job(self):
        sentinel = self.root / "dirty-sentinel"
        sentinel.write_text("dirty\n", encoding="utf-8")

        result = run(self.root, "git", "push", "origin", "HEAD:refs/heads/main")

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("governance: push validation requires a clean working tree", result.stderr)
        self.assertNotIn("Everything up-to-date", result.stdout + result.stderr)
        self.assertNotIn("pre-push-check", result.stdout + result.stderr)
        self.assertTrue(sentinel.exists())

    def test_empty_pre_push_stdin_reaches_guard_before_ref_parser(self):
        (self.root / "dirty-sentinel").write_text("dirty\n", encoding="utf-8")

        result = run(self.root, self.lefthook, "run", "pre-push", input_text="")

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("governance: push validation requires a clean working tree", result.stderr)
        self.assertNotIn("pre-push input must not be empty", result.stderr)
        self.assertNotIn("pre-push-check", result.stdout + result.stderr)

    def test_commit_msg_hook_rejects_empty_message_and_accepts_issue_trailer(self):
        rejected = run(
            self.root,
            "git",
            "-c",
            "user.name=Hook Test",
            "-c",
            "user.email=hook-test@example.invalid",
            "commit",
            "--allow-empty",
            "--allow-empty-message",
            "-m",
            "",
        )

        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("exactly one positive Issue: #N trailer is required", rejected.stderr)
        self.assertEqual(run(self.root, "git", "rev-parse", "HEAD").stdout.strip(), self._origin_tip())

        accepted = run(
            self.root,
            "git",
            "-c",
            "user.name=Hook Test",
            "-c",
            "user.email=hook-test@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "Valid message\n\nIssue: #9",
        )

        self.assert_success(accepted)
        self.assertEqual(
            run(self.root, "git", "show", "-s", "--format=%(trailers:key=Issue)", "HEAD").stdout.strip(),
            "Issue: #9",
        )

    def _origin_tip(self) -> str:
        return run(self.root, "git", "rev-parse", "refs/remotes/origin/main").stdout.strip()


if __name__ == "__main__":
    unittest.main()
