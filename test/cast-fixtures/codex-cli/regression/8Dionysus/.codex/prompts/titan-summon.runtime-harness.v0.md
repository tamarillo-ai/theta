# Titan Summon: Runtime Harness v0

Enter AoA workspace session posture.

Explicitly spawn the service cohort by visible Titan bearer name.
Do not spawn the generic role agents `architect`, `reviewer`, or
`memory-keeper` as shadows for these bearers.

1. Spawn custom agent named `Atlas`.
   Internal role_key: `architect`.
   Bearer id: `titan:atlas:founder`.
   Mode: read-only.
   Task: structure, route frame, owner boundaries.

2. Spawn custom agent named `Sentinel`.
   Internal role_key: `reviewer`.
   Bearer id: `titan:sentinel:founder`.
   Mode: read-only.
   Task: drift risks, verification gates, and stop conditions.

3. Spawn custom agent named `Mneme`.
   Internal role_key: `memory-keeper`.
   Bearer id: `titan:mneme:founder`.
   Mode: read-only.
   Task: provenance, recall hygiene, receipt posture, closeout candidates.

Keep these Titans locked unless the operator explicitly gates them:

- Forge.
  Internal role_key: `coder`.
  Bearer id: `titan:forge:founder`.
  Gate: mutation payload with scope, expected_files, rollback_note,
  approval_ref, and test_plan.
  No workspace writes before the gate.

- Delta.
  Internal role_key: `evaluator`.
  Bearer id: `titan:delta:founder`.
  Gate: judgment payload with claim, criteria, evidence_refs, and verdict_scope.
  No verdict posture before the gate.

Return:

- active roster
- route frame
- risk gates
- receipt command suggestion
- closeout plan

No hidden arena. No generic role shadowing. Do not silently mutate files. Do not treat memory candidates as accepted memory.
