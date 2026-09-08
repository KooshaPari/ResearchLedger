import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from path_safety import resolve_under


class PathSafetyTests(unittest.TestCase):
    def test_relative_path_is_resolved_beneath_declared_root(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.assertEqual(
                resolve_under(Path("private/output.json"), root),
                root.resolve() / "private/output.json",
            )

    def test_traversal_and_symlink_escapes_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "root"
            outside = Path(temporary) / "outside"
            root.mkdir()
            outside.mkdir()
            (root / "link").symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, "PATH_OUTSIDE_ALLOWED_ROOT"):
                resolve_under(Path("../outside/data.json"), root)
            with self.assertRaisesRegex(ValueError, "PATH_OUTSIDE_ALLOWED_ROOT"):
                resolve_under(Path("link/data.json"), root)

    def test_wave6_runner_temp_private_and_repo_public_roots_validate_before_network(self):
        script = Path(__file__).with_name("collect_emergent_garden_comments.py")
        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = Path(temporary)
            runner_temp = temporary_root / "runner-temp"
            workspace = temporary_root / "workspace"
            runner_temp.mkdir()
            workspace.mkdir()
            inventory = workspace / "inventory.json"
            inventory.write_text(json.dumps({"videos": []}))
            result = subprocess.run(
                [
                    sys.executable,
                    str(script),
                    "--private",
                    str(runner_temp / "private"),
                    "--private-root",
                    str(runner_temp),
                    "--public",
                    str(runner_temp / "public"),
                    "--public-root",
                    str(runner_temp),
                    "--inventory",
                    str(inventory),
                    "--inventory-root",
                    str(workspace),
                ],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("KEY_MISSING", result.stderr)

    def test_assessment_uses_separate_fixture_and_output_roots(self):
        script = Path(__file__).with_name("assess_research_evidence.py")
        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = Path(temporary)
            fixture_root = temporary_root / "fixtures"
            output_root = temporary_root / "public"
            fixture_root.mkdir()
            output_root.mkdir()
            fixture = fixture_root / "cases.json"
            fixture.write_text(json.dumps({"cases": []}))
            output = output_root / "result.json"
            result = subprocess.run(
                [
                    sys.executable,
                    str(script),
                    str(fixture),
                    "--fixtures-root",
                    str(fixture_root),
                    "--output",
                    str(output),
                    "--output-root",
                    str(output_root),
                ],
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(output.is_file())


if __name__ == "__main__":
    unittest.main()
