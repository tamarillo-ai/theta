# Exploration Test Plan: <task-id>

- `skill`: exploration-test-planning
- `status`: draft
- `source_plan`: `./plan.md`
- `owner_agent`: `exploration_test_planner`
- `return_to`: `exploration_test_lane`

## Source Inputs

- `request_summary`:
- `existing_artifacts`:
- `human_constraints`:
- `excluded_targets`:

## Observation Scope

- `targets`:
- `entry_points`:
- `state_or_log_targets`:
- `out_of_scope`:

## Exploration Viewpoints

- `failure_viewpoints`:
- `state_transition_viewpoints`:
- `recovery_viewpoints`:
- `permission_or_trust_viewpoints`:
- `log_viewpoints`:

## Test Data Policy

- `required_inputs`:
- `required_state`:
- `reusable_fixtures`:
- `data_constraints`:

## Stop Conditions

- `sufficient_evidence`:
- `not_reproducible`:
- `environment_blocker`:
- `needs_human_decision`:

## Planning Blockers

- `blocker_id`:
- `summary`:
- `reason`:
- `required_correction`:
- `status`: `open | fixed`

## Correction History

- `blocker_id`:
- `correction`:
- `remaining_gap`:
- `evidence_ref`:

## Output

- `decision`: `complete | needs_correction | stopped`
- `evidence_refs`:
- `missing_info`:
- `planning_blockers_open`:
- `next_artifact`: `./exploration-test-data.md`
