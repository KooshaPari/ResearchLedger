import os
import re
import subprocess
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

    def test_workflow_contracts_match_for_dispatch_and_call(self):
        workflow = Path(".github/workflows/retroactive-sweep.yml").read_text()
        dispatch = re.search(r"  workflow_dispatch:\n    inputs:\n(.*?)(?=  workflow_call:)", workflow, re.S)
        call = re.search(r"  workflow_call:\n    inputs:\n(.*?)(?=permissions:)", workflow, re.S)
        self.assertIsNotNone(dispatch)
        self.assertIsNotNone(call)
        for block in (dispatch.group(1), call.group(1)):
            for name, kind, default in (
                ("since_days", "number", "7"),
                ("max_prs", "number", "50"),
                ("dry_run", "boolean", "false"),
                ("repos", "string", '""'),
            ):
                self.assertRegex(
                    block,
                    rf"{name}:\n(?:.*\n)*?        type: {kind}\n        default: {default}",
                )

    def test_resolver_preserves_zero_and_false_input_values(self):
        workflow = Path(".github/workflows/retroactive-sweep.yml").read_text()
        self.assertIn('echo "lookback=${LOOKBACK:-30}"', workflow)
        self.assertIn('echo "max_prs=${MAX_PRS:-20}"', workflow)
        self.assertIn('echo "dry_run=${DRY_RUN:-true}"', workflow)
        result = subprocess.check_output(
            ["bash", "-c", 'printf "%s,%s,%s" "${LOOKBACK:-30}" "${MAX_PRS:-20}" "${DRY_RUN:-true}"'],
            env={**os.environ, "LOOKBACK": "0", "MAX_PRS": "0", "DRY_RUN": "false"},
            text=True,
        )
        self.assertEqual(result, "0,0,false")
