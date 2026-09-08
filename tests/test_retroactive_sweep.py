import unittest
from pathlib import Path

from scripts.retroactive_sweep import parse_repositories


class RepositoryInputTests(unittest.TestCase):
    CURRENT = "KooshaPari/ResearchLedger"

    def test_empty_uses_current_repository(self):
        self.assertEqual(parse_repositories("", self.CURRENT), [self.CURRENT])

    def test_json_list_is_validated_and_deduplicated(self):
        self.assertEqual(
            parse_repositories('["KooshaPari/ResearchLedger", "octo/example", "octo/example"]', self.CURRENT),
            ["KooshaPari/ResearchLedger", "octo/example"],
        )

    def test_plain_scalar_repository_remains_compatible(self):
        self.assertEqual(parse_repositories("octo/example", self.CURRENT), ["octo/example"])

    def test_malformed_json_wrong_type_and_invalid_identifier_fail_before_api(self):
        for value in ('["octo/example"', '{"repo":"octo/example"}', '["not/a/repo/extra"]'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                parse_repositories(value, self.CURRENT)

    def test_workflow_defaults_preserve_zero_and_false(self):
        workflow = Path(".github/workflows/retroactive-sweep.yml").read_text()
        self.assertIn('echo "lookback=${LOOKBACK:-30}"', workflow)
        self.assertIn('echo "dry_run=${DRY_RUN:-true}"', workflow)

