import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from cleanup_policy import CleanupPolicyError, REGISTRY_TRUST_LIMITATION, local_plan, parse_release_payload, registry_version, remote_plan, require_tag


class CleanupPolicyTests(unittest.TestCase):
    def test_release_payload_is_strict(self):
        raw = json.dumps({"tag_name": "v0.1.0", "draft": False, "prerelease": False, "published_at": "2026-10-20T00:00:00Z", "html_url": "https://github.com/acme/kle/releases/tag/v0.1.0"})
        self.assertEqual(parse_release_payload(raw, "acme/kle", "v0.1.0")["tag_name"], "v0.1.0")
        for key in ("draft", "prerelease", "published_at", "html_url"):
            value = json.loads(raw)
            value[key] = True if key in {"draft", "prerelease"} else ""
            with self.subTest(key=key), self.assertRaises(CleanupPolicyError):
                parse_release_payload(json.dumps(value), "acme/kle", "v0.1.0")

    def test_exact_tag(self):
        for tag in ("v0.1.0", "v12.3.45"):
            self.assertEqual(require_tag(tag), tag)
        for tag in ("v0.1", "v0.1.0-rc1", "main", "v0.1.0 "):
            with self.subTest(tag=tag), self.assertRaises(CleanupPolicyError):
                require_tag(tag)

    def test_registry_version_strips_only_verified_v_prefix(self):
        self.assertEqual(registry_version("v0.1.0"), "0.1.0")
        with self.assertRaises(CleanupPolicyError):
            registry_version("0.1.0")
        self.assertIn("exit=0", REGISTRY_TRUST_LIMITATION)

    def test_remote_requires_both_ancestry_facts(self):
        graph = {("a", "tag"), ("a", "default"), ("b", "tag"), ("c", "default")}
        plan = remote_plan([("release/a", "a"), ("release/b", "b")], "tag", "main", "default", lambda child, parent: (child, parent) in graph)
        self.assertEqual([item.action for item in plan.decisions], ["delete", "retain"])
        self.assertTrue(plan.residual)

    def test_remote_protects_release_named_default_and_retains_advanced_tip(self):
        graph = {("old", "tag"), ("old", "default")}
        plan = remote_plan([("release/main", "old"), ("release/advanced", "new")], "tag", "release/main", "default", lambda child, parent: (child, parent) in graph)
        self.assertEqual([item.action for item in plan.decisions], ["retain", "retain"])
        self.assertEqual(plan.decisions[0].reason, "protected default branch")

    def test_local_protects_bound_default_current_and_unmerged(self):
        plan = local_plan([("main", "yes"), ("release/current", "yes"), ("release/bound", "yes"), ("release/unmerged", "no"), ("release/old", "yes")], {"release/bound"}, "main", "release/current")
        self.assertEqual([item.action for item in plan.decisions], ["retain", "retain", "retain", "retain", "delete"])
        ordinary = local_plan([("main", "yes"), ("feature/x", "yes")], set(), "main", "main")
        self.assertFalse(ordinary.residual)


if __name__ == "__main__":
    unittest.main()
