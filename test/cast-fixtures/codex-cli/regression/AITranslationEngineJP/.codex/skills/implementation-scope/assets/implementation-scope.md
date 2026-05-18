# Implementation Scope: <task-id>

- `skill`: implementation-scope
- `status`: draft
- `source_plan`: `./plan.md`
- `human_review_status`:
- `approval_record`:
- `codex_entry`: `.codex/skills/implement-lane/SKILL.md`
- `handoff_runtime`: `codex`
- `architecture_reference`: `docs/architecture.md`

## Source Artifacts

- `ui_design`: `./ui-design.md` または `N/A`
- `ui_agent_browser_review`: `./ui-design.md#agent-browser-review` または `N/A`
- `scenario_design`: `./scenario-design.md`
- `detail_requirement_coverage`: `./scenario-design.requirement-coverage.json`
- `human_decision_questionnaire`: `./scenario-design.questions.md`

## Fixed Decisions

- human review 済みの判断だけを書く
- frontend handoff がある場合は、承認済み `ui-design.md` を source にする
- `needs_human_decision`: `0`
- 承認済み詳細要求タイプと質問票回答だけを handoff source にする
- downstream handoff が依存する public seam は各実装 handoff の完了条件として固定する
- secret を扱う handoff は参照値、secret 本体、secret 解決責務層、出力禁止値を分ける
- backend、frontend、統合境界 は原則として別 handoff に分ける
- `E2E` は UI 人間操作起点だけを指す
- `APIテスト` は public seam 起点の system-level test とする

## Ready Waves

| ready_wave | handoffs | depends_on_done_before_start | parallel_pairs | blockers |
| --- | --- | --- | --- | --- |
| `wave-1` | `<handoff_id>` | `なし` | `<handoff_id> <-> <handoff_id>` または `なし` | `<parallel_blockers>` または `なし` |

## Handoffs

### `handoff_id`:

- `implementation_target`:
- `implementation_artifact`: `backend 実装 | frontend 実装 | 統合境界実装 | シナリオテスト | 単体テスト`
- `implementation_skill`: `implement-backend | implement-frontend | implement-integration | tests-scenario | tests-unit`
- `frontend_required_sources`:
  - `ui_design`: `./ui-design.md` または `N/A`
  - `ui_agent_browser_review`: `./ui-design.md#agent-browser-review` または `N/A`
- `secret_boundary`:
  - `status`: `required | not_required`
  - `reference_values_allowed_in_ui_dto_read_model`:
  - `secret_values_for_provider_external_api_internal_auth`:
  - `secret_resolution_owner_layer`:
  - `forbidden_outputs`:
- `owned_scope`:
- `depends_on`:
- `execution_group`:
- `ready_wave`:
- `parallelizable_with`:
- `parallel_blockers`:
- `first_action`:
- `validation_commands`:
- `completion_signal`:
- `acceptance_test`: `required`
- `execution_test_classification`: `APIテスト | UI人間操作E2E | lower-level only`
- `execution_stage`: `実装後 | final validation`
- `notes`:
  - backend と frontend は必ず別 handoff に分ける。UI がある task では frontend handoff を backend handoff より先に置く。
  - frontend handoff では、承認済み `ui-design.md` を必須 source にする。
  - frontend handoff では、承認済み UI 要件契約の主要区画、導線、状態表示を維持する完了条件を書く。
  - API / Wails / DTO / gateway / adapter contract の接続と実画面確認は `統合境界実装` handoff に分ける。
  - `implementation_skill` は `implementation_artifact` と一致させ、Codex implementation lane が読む skill を一意にする。
  - `シナリオテスト` と `単体テスト` は実装成果物の完了後に別 handoff として作り、依存対象が揃った後は並列実行できる。
  - `APIテスト` と `UI人間操作E2E` は実装後の `シナリオテスト` で証明する。
  - secret を扱う handoff では、UI / DTO / read model に出してよい参照値と、provider / external API / internal auth に渡す secret 本体を `secret_boundary` に分けて書く。
  - `credential_ref`、`secret_ref`、`api_key`、`token` などの field 名がある場合は、参照値と secret 本体を同じ値として扱わない。
  - `forbidden_outputs` には log、error summary、audit、request capture、URL、DTO、UI、read model に出してはいけない値を書く。
  - `execution_group` は `wave-1`、`wave-2`、`wave-3` のように必要な数だけ作る。同じ wave 内でも `parallelizable_with` に列挙しない handoff は並列実行しない。
  - `ready_wave` は Ready Waves 表と一致させる。Codex は最小番号の実行可能 wave から開始する。
  - `first_action` は Codex implementation lane が最初に閉じる 1 clause だけを書く。path、symbol または対象単位、変更種別、対応する `completion_signal` clause を含める。
  - 並列不可の理由は `parallel_blockers` に `depends_on`、`owned_scope_overlap`、`shared_contract_change`、`validation_owner_ambiguous`、`backend_frontend_order`、`broad_gate_shared` のいずれかで書く。
  - 必要な場合だけ `本番経路` を書く。`本番経路` は実行時に通る public API / DTO / controller / UI entry / persistence path を指し、domain 固有知識はここへ一般例として増やさない。

## Completion Packet

Codex implementation lane は完了時に次を返す。

- `completed_handoffs`
- `touched_files`
- `implemented_scope`
- `test_results`
- `implementation_investigation`
- `ui_evidence`
- `final_validation_result`
- `codex_review_result`
- `coverage_gate_result`
- `sonar_gate_result`: 互換 field 名。意味は repo-local Sonar issue gate であり、Sonar サーバ側 Quality Gate ではない。
- `harness_gate_result`: system test が Wails / sandbox / OS 権限で止まる場合は `FAIL_ENVIRONMENT` とし、blocked reason、再実行環境、再実行コマンドを残す。
- `residual_risks`
- `completion_evidence`: Codex 側 `work_reporter` が読む実装事実。report 文面ではなく、completed_handoffs、touched_files、validation、residual、blocked reason、人間が次に見るべき場所を含める。
- `telemetry_events`: `runtime: codex` の response event。速度や欠落は次回改善用であり、初期 close 判定には使わない。
- `docs_changes: none`
