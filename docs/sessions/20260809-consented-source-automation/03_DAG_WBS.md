# DAG and Work Breakdown

```text
consent registry
      |
      +--> scope validator --> acquisition adapters --> artifact store --> claim spans
      |
      +--> consent UI/audit

GitHub auth --> identity binding --> starred import
LinkedIn probe --> export/permalink/manual import
```

1. Define/persist consent and scope contract.
2. Build policy-safe adapters and auth boundary.
3. Persist artifacts and claim spans.
4. Add acceptance and debug-only desktop checks.

Critical path: 1 -> 2 -> 3 -> 4. Live provider verification remains a separate gate.
