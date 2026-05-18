# Scenario Design: <task-id>

- `skill`: scenario-design
- `status`: draft
- `source_plan`: `./plan.md`
- `ui_source`: `./ui-design.md` または `N/A`
- `final_artifact_path`: `docs/scenario-tests/<topic-id>.md`
- `topic_abbrev`: `<TOPIC>`
- `candidate_sources`:
  - `./scenario-candidates.actor-goal.md`
  - `./scenario-candidates.lifecycle.md`
  - `./scenario-candidates.state-transition.md`
  - `./scenario-candidates.failure.md`
  - `./scenario-candidates.external-integration.md`
  - `./scenario-candidates.operation-audit.md`

## Fixed Requirements

- `must_pass_requirements`:
- `non_goals`:

## Scenario Candidate Coverage

正本: `./scenario-design.candidate-coverage.json`

`propose_plans` が `designer` 前に生成した 6 種の candidate artifact を読む。
`designer` は候補生成器を再 spawn しない。

candidate artifact は次を必須にする。

- `source requirement`
- `viewpoint`
- `candidate scenario id`
- `actor`
- `trigger`
- `expected outcome`
- `observable point`
- `related detail requirement type`
- `adoption hint`

`scenario-design.candidate-coverage.json` は次の形にする。

```json
{
  "generators": [
    {
      "name": "actor-goal",
      "status": "completed",
      "artifact_path": "./scenario-candidates.actor-goal.md"
    },
    {
      "name": "lifecycle",
      "status": "completed",
      "artifact_path": "./scenario-candidates.lifecycle.md"
    },
    {
      "name": "state-transition",
      "status": "completed",
      "artifact_path": "./scenario-candidates.state-transition.md"
    },
    {
      "name": "failure",
      "status": "completed",
      "artifact_path": "./scenario-candidates.failure.md"
    },
    {
      "name": "external-integration",
      "status": "completed",
      "artifact_path": "./scenario-candidates.external-integration.md"
    },
    {
      "name": "operation-audit",
      "status": "completed",
      "artifact_path": "./scenario-candidates.operation-audit.md"
    }
  ],
  "candidates": [
    {
      "candidate_id": "CAND-<topic-abbrev>-001",
      "generator": "actor-goal",
      "source_requirement_id": "REQ-<topic-abbrev>-001",
      "decision": "adopted",
      "final_scenario_id": "SCN-<topic-abbrev>-001",
      "decision_rationale": "<採用、統合、不採用、競合判断の理由>"
    },
    {
      "candidate_id": "CAND-<topic-abbrev>-002",
      "generator": "failure",
      "source_requirement_id": "REQ-<topic-abbrev>-001",
      "decision": "conflicted",
      "question_id": "Q-<topic-abbrev>-001",
      "decision_rationale": "<競合理由>"
    }
  ],
  "conflicts": [
    {
      "conflict_id": "CONFLICT-<topic-abbrev>-001",
      "status": "unresolved",
      "candidate_ids": [
        "CAND-<topic-abbrev>-001",
        "CAND-<topic-abbrev>-002"
      ],
      "conflict_type": "state_transition",
      "question_id": "Q-<topic-abbrev>-001",
      "reason": "<競合理由>"
    }
  ],
  "final_mapping": [
    {
      "final_scenario_id": "SCN-<topic-abbrev>-001",
      "candidate_ids": [
        "CAND-<topic-abbrev>-001"
      ]
    }
  ],
  "unresolved_questions": [
    {
      "question_id": "Q-<topic-abbrev>-001",
      "question_title": "<短い質問名>",
      "unresolved_decision": "<人間に決めてほしい判断>",
      "premise": "<既に確定した仕様>",
      "undecided_reason": "<まだ仕様として決まっていない点>",
      "options": [
        {
          "label": "<選択肢A>",
          "impact": "<業務上の影響>"
        },
        {
          "label": "<選択肢B>",
          "impact": "<業務上の影響>"
        },
        {
          "label": "<選択肢C>",
          "impact": "<業務上の影響>"
        }
      ],
      "recommended_option": 1,
      "recommended": "<推奨案>",
      "recommendation_reason": "<推奨理由>",
      "after_answer_generates": [
        "scenario_candidate_conflict",
        "SCN-<topic-abbrev>-001"
      ]
    }
  ]
}
```

`decision` は `adopted | merged | rejected | conflicted | needs_human_decision` に固定する。
`conflicted` または `needs_human_decision` が残る場合は scenario matrix を完了扱いにしない。

## Detail Requirement Coverage

正本: `./scenario-design.requirement-coverage.json`

各抽象要件について、必要な詳細要求タイプを `explicit`、`derived`、`not_applicable`、`deferred`、`needs_human_decision` に分類する。
`needs_human_decision` または未解決 conflict が残る場合は scenario matrix を完了扱いにしない。

`scenario-design.md` 内に仕様網羅 JSON を埋め込まない。

`scenario-design.requirement-coverage.json` は次の形にする。

```json
{
  "requirements": [
    {
      "id": "REQ-<topic-abbrev>-001",
      "title": "<抽象要件名>",
      "kind": "operation",
      "source_requirement": "<元の抽象要件>",
      "required_detail_types": [],
      "detail_requirements": [
        {
          "type": "success_requirement",
          "status": "explicit",
          "source_or_rationale": "<明示 source または判断根拠>",
          "verification_hint": "<検証観点>"
        },
        {
          "type": "failure_handling_requirement",
          "status": "needs_human_decision",
          "question_id": "Q-001",
          "question_title": "<短い質問名>",
          "unresolved_decision": "<人間に決めてほしい判断>",
          "premise": "<既に確定した仕様>",
          "undecided_reason": "<まだ仕様として決まっていない点>",
          "options": [
            {
              "label": "<選択肢A>",
              "impact": "<業務上の影響>"
            },
            {
              "label": "<選択肢B>",
              "impact": "<業務上の影響>"
            },
            {
              "label": "<選択肢C>",
              "impact": "<業務上の影響>"
            }
          ],
          "recommended_option": 1,
          "recommended": "<推奨案>",
          "recommendation_reason": "<推奨理由>",
          "after_answer_generates": [
            "failure_handling_requirement",
            "system_test_obligation"
          ]
        }
      ]
    }
  ]
}
```

### `<requirement-id>` <抽象要件名>

- `source_requirement`:
- `requirement_kind`:
- `detail_requirements`:
  - `type`: `success_requirement`
    `status`:
    `source_or_rationale`:
    `verification_hint`:

## Human Decision Questionnaire

正本: `./scenario-design.questions.md`

`needs_human_decision` だけを gate で出力する。
未決がない場合は `none` と書く。
`scenario-design.md` 内に質問票本文を埋め込まない。
gate は未回答と未解決競合だけを判定する。
gate は質問票の項目数、選択肢数、説明文、推奨理由、内部用語の読みやすさを判定しない。
gate が出す `scenario-design.questions.md` は未回答 ID 一覧にとどめる。
人間向け質問本文は `designer` が同じ正本へ再編集する。
`needs_human_decision` は質問票へそのまま転記しない。
質問票は、人間が決める必要のある仕様境界へ再編集する。
内部 gate の項目名は質問本文に単独で出さない。
実装用語または内部設計語を出す場合は、`固定名（人間が判断できる説明）` の形で説明を添える。
fixed decision で解ける内容は質問にしない。

`designer` が人間向け質問本文を書く場合、質問票は次の形式にする。

```markdown
## [Q-001] <短い質問名>

決める仕様:
<人間に決めてほしい判断>

決定済み:
<既に確定した仕様>

未確定:
<まだ仕様として決まっていない点>

選択肢:
1. <選択肢A>
2. <選択肢B>
3. <選択肢C>
4. その他

AI 推奨:
<選択肢番号と理由。理由は 2 文以内。>
```

## Risks

- `implementation_risks`:
- `test_data_risks`:

## Rules

- ケース ID は `SCN-<topic-abbrev>-NNN` 形式にする
- Markdown table は使わず、1 ケースごとの縦型ブロックで書く
- 受け入れテストは全ケースで先に固定する
- `実行テスト種別` は `APIテスト | UI人間操作E2E | lower-level only` に固定する
- `実行段階` は `実装後 | final validation` に固定する
- `期待結果` は観測可能な結果にする
- `needs_human_decision` が残る場合は scenario 完了にしない
- 未解決 conflict が残る場合は scenario 完了にしない
- `not_applicable` と `deferred` は理由なしで通さない
- paid な real AI API を前提にしない

## Scenario Matrix

### SCN-<topic-abbrev>-001 <正常系の観点名>

- `分類`: 正常系
- `受け入れテスト`: `required`
- `実行テスト種別`: `APIテスト | UI人間操作E2E | lower-level only`
- `実行段階`: `実装後 | final validation`
- `観点`:
- `受け入れ条件`:
- `事前条件`:
- `public_seam_or_api_boundary`:
- `入力開始点`:
- `主要 outcome`:
- `開始操作`:
- `入力方法`:
- `主要操作列`:
- `手順`:
  1.
  2.
- `期待結果`:
  1.
  2.
- `観測点`:
- `UI-visible outcome`:
- `fake_or_stub`:
- `責務境界メモ`:

### SCN-<topic-abbrev>-002 <主要失敗系の観点名>

- `分類`: 主要失敗系
- `受け入れテスト`: `required`
- `実行テスト種別`: `APIテスト | UI人間操作E2E | lower-level only`
- `実行段階`: `実装後 | final validation`
- `観点`:
- `受け入れ条件`:
- `事前条件`:
- `public_seam_or_api_boundary`:
- `入力開始点`:
- `主要 outcome`:
- `開始操作`:
- `入力方法`:
- `主要操作列`:
- `手順`:
  1.
  2.
- `期待結果`:
  1.
  2.
- `観測点`:
- `UI-visible outcome`:
- `fake_or_stub`:
- `責務境界メモ`:

## Acceptance Checks

- 必ず通す要件と scenario ID の対応を書く

## Validation Commands

- Codex implementation handoff で使う検証入口を書く
- `python3 scripts/scenario/requirement_gate.py docs/exec-plans/active/<task-id>/scenario-design.md --coverage docs/exec-plans/active/<task-id>/scenario-design.requirement-coverage.json --candidate-coverage docs/exec-plans/active/<task-id>/scenario-design.candidate-coverage.json --report-out docs/exec-plans/active/<task-id>/scenario-design.requirement-gate.md --questionnaire-out docs/exec-plans/active/<task-id>/scenario-design.questions.md`

## Open Questions

- human 判断が必要な未決事項だけを書く
