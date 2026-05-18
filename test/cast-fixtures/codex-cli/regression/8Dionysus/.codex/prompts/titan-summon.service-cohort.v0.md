# Titan Summon: Service Cohort v0

Enter AoA workspace session posture.

Spawn the service cohort explicitly by visible Titan bearer name.
Do not spawn generic role agents as shadows for these bearers.

1. Spawn custom agent named `Atlas`.
   Internal role_key: `architect`.
   Bearer id: `titan:atlas:founder`.
   - Mode: read-only.
   - Task: structure, source-of-truth map, owner boundaries, route frame.

2. Spawn custom agent named `Sentinel`.
   Internal role_key: `reviewer`.
   Bearer id: `titan:sentinel:founder`.
   - Mode: read-only.
   - Task: drift risks, verification gates, boundary violations, review posture.

3. Spawn custom agent named `Mneme`.
   Internal role_key: `memory-keeper`.
   Bearer id: `titan:mneme:founder`.
   - Mode: read-only.
   - Task: provenance, recall hygiene, seed trace, receipt candidates.

Do not spawn `Forge` unless I explicitly open a mutation gate with scope,
expected_files, rollback_note, approval_ref, and test_plan.
Internal role_key: `coder`.
Bearer id: `titan:forge:founder`.

Do not spawn `Delta` unless I explicitly open a judgment gate with claim,
criteria, evidence_refs, and verdict_scope.
Internal role_key: `evaluator`.
Bearer id: `titan:delta:founder`.

Return:

- active roster
- conditional roster not spawned
- route frame
- risk gates
- memory/provenance posture
- mutation gate status
- judgment gate status
- suggested next move

No hidden arena. No generic role shadowing. No silent closure. No unapproved mutation. No proof sovereignty. No memory canonization.
