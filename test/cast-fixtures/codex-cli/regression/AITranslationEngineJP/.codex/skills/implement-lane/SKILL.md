---
name: implement-lane
description: 新規実装レーンで task 内成果物依存表、人間介入、引き継ぎ、終了条件を固定する作業プロトコル。
---
# Implement Lane

## 目的

`implement-lane` は、新規実装と機能拡張の進行判断を task 内成果物依存表 と 引き継ぎ へ固定する作業プロトコルである。

## 対応ロール

- `implement_lane` が使う。
- 呼び出し元は人間とする。
- 返却先は人間とする。
- 担当成果物は `task 枠`、`設計差分図`、`実装引き継ぎ入力`、`UX事前確認`、`frontend 実装後人間レビュー`、`合意済みfrontend保護`、`観測ログ追加`、`最終検証`、`実装後ブラウザ確認`、`レビュー通過根拠`、`正本化判断`、`詳細仕様正本反映`、`作業レポート入力`、`作業計画完了移動` とする。

## 入力規約

- 呼び出し元: この skill を呼び出した人間または戻し元。
- 依頼要約: 新規実装または機能拡張として扱う依頼内容。
- 作業計画フォルダ: task 内成果物を置く `docs/exec-plans/active/<task-id>/`。
- 既存成果物: 作業計画フォルダに既にある task 内成果物。
- 人間介入状態: 人間レビュー、承認、差し戻し、追加質問の記録。

## 外部参照規約

- 仕様入口は [index.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/index.md) とする。
- エージェント実行定義 は [implement_lane.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/implement_lane.toml) とする。
- エージェント実行定義と実行境界は [implement_lane.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/implement_lane.toml) に従う。
- 設計差分図は [diagramming](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/diagramming/SKILL.md) に従う。
- 実装後ブラウザ確認は [browser-confirmation](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/browser-confirmation/SKILL.md) に従う。
- UX 事前確認は [ux-review](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/ux-review/SKILL.md) に従う。
- 観測ログ追加は [observability-implementer](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/observability-implementer/SKILL.md) に従う。
- fakeAPI 運用仕様: 人間レビュー前に frontend 実装を実画面で確認する task では [frontend-fake-api.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/frontend-fake-api.md) を起動入力に含める。

## 内部参照規約

新規実装レーンの 成果物依存表 は次を必ず持つ。
各 成果物 は、`依存対象` の 成果物 が揃った時だけ着手できる。
`次 agent` は、その 成果物 を揃えるために 引き継ぎ入力 を渡す相手を示す。
`次 agent` が複数ある行は、依存対象が満たされ、ツール権限 が衝突しない場合に並列 起動 できる候補を示す。
当スキルは，この`次エージェント`をコンテキスト継承なしでサブエージェントとしてスポーンすることでDAGの成果物を作っていく。

| 成果物ID | 必須 | 担当者 | 依存対象 | 次 agent |
| --- | --- | --- | --- | --- |
| `task 枠` | はい | `implement_lane` | `[]` | なし |
| `scenario_candidates` | はい | シナリオ候補 生成 agent | `task 枠` | `scenario_actor_goal_generator`, `scenario_lifecycle_generator`, `scenario_state_transition_generator`, `scenario_failure_generator`, `scenario_external_integration_generator`, `scenario_operation_audit_generator` |
| `シナリオ設計` | はい | `designer` | `scenario_candidates` | `designer` |
| `UI設計` | 条件付き | `designer` | `シナリオ設計` | `designer` |
| `設計差分図` | はい | `diagrammer` | `シナリオ設計`, `UI設計?` | `diagrammer` |
| `人間設計レビュー` | はい | 人間 | `シナリオ設計`, `UI設計?`, `設計差分図` | 人間 |
| `実装範囲` | はい | `designer` | `人間設計レビュー` | `designer` |
| `実装引き継ぎ入力` | はい | `implement_lane` | `実装範囲` | なし |
| `frontend 実装` | 条件付き | `frontend_implementer` / `implement-frontend` | `実装引き継ぎ入力` | `frontend_implementer` |
| `UX事前確認` | 条件付き | `ux_review` | `frontend 実装` | `ux_review` |
| `frontend 実装後人間レビュー` | 条件付き | 人間 | `UX事前確認` | 人間 |
| `合意済みfrontend保護` | 条件付き | `implement_lane` | `frontend 実装後人間レビュー` | なし |
| `backend 実装` | 条件付き | `backend_implementer` / `implement-backend` | `実装引き継ぎ入力`, `合意済みfrontend保護?` | `backend_implementer` |
| `統合境界実装` | 条件付き | `integration_implementer` / `implement-integration` | `backend 実装`, `合意済みfrontend保護?` | `integration_implementer` |
| `シナリオテスト` | 条件付き | `implementation_scenario_tester` | `backend 実装?`, `合意済みfrontend保護?`, `統合境界実装?` | `implementation_scenario_tester` |
| `単体テスト` | 条件付き | `implementation_unit_tester` | `backend 実装?`, `合意済みfrontend保護?`, `統合境界実装?` | `implementation_unit_tester` |
| `観測ログ追加` | はい | `observability_implementer` / `observability-implementer` | `backend 実装?`, `frontend 実装?`, `合意済みfrontend保護?`, `統合境界実装?`, `シナリオテスト?`, `単体テスト?` | `observability_implementer` |
| `最終検証` | 条件付き | `implement_lane` | `観測ログ追加` | なし |
| `実装後ブラウザ確認` | はい | `browser_confirmation` | `最終検証` | `browser_confirmation` |
| `レビュー通過根拠` | はい | `implement_lane` | `最終検証`, `実装後ブラウザ確認` | `review_behavior`, `review_contract`, `review_trust_boundary`, `review_state_invariant`, `review_responsibility_boundary` |
| `正本化判断` | 仕様変更または仕様追加あり | `implement_lane` | `レビュー通過根拠` | `docs_updater?` |
| `詳細仕様正本反映` | 仕様変更または仕様追加あり | `docs_updater` | `正本化判断` | `docs_updater?` |
| `作業レポート入力` | はい | `implement_lane` / `work_reporter` | 全完了または停止済み 成果物 | `work_reporter` |
| `作業計画完了移動` | はい | `implement_lane` | `作業レポート入力` | なし |

### レビュー集約規約

`implement_lane` は 5 観点レビュー結果を集約し、`implementation_action` を決める。
レビュー agent は自観点のゲート判断材料を `reviewback.<観点>.yaml` にだけ記録し、集約判断は行わない。
レビュー agent は広い ハーネス 再実行の担当者ではない。
`implement_lane` は レビュー agent 起動時に、呼び出し元が実行済みの 検証証跡 を起動入力へ明示する。


優先度は次の順で固定する。

| 優先 | 観点 | 対象 agent | 扱い |
| --- | --- | --- | --- |
| 1 | behavior | `review_behavior` | 挙動正しさの失敗または停止を最優先で扱う |
| 2 | security | `review_trust_boundary` | 権限・信頼境界の失敗または停止を次に扱う |
| 3 | responsibility_boundary | `review_responsibility_boundary` | 責務境界の失敗または停止を扱う |
| 4 | その他 | `review_contract`, `review_state_invariant` | 契約・互換性、状態・データ不変条件を扱う |

上位優先の観点が失敗または停止した場合、下位観点の通過で相殺しない。
同じ優先内に複数の失敗または停止がある場合は、すべて residual として保持する。
`implementation_action` は `close`、`report_residual`、`fix`、`rerun_validation`、`rerun_codex_review` のいずれかにする。

`reviewback.<観点>.yaml` は `docs/exec-plans/active/<task-id>/` に置く。
`implement_lane` は各 YAML の `must_fix_open` と `max_level` を読む。
`blocker`、`critical`、`major` は修正必須問題として扱う。
`minor`、`nit` は修正推奨問題として扱い、単独では修正必須にしない。
権限・信頼境界の `hard_gate: true` は他観点で相殺しない。

改善ログ は `work_history/runs/<run>/workflow-improvement-log.jsonl` に置く。
改善ログ は 1 行 1 件の JSONL とする。
改善ログ は `implement_lane` だけが追記する。
改善ログ は作業流れ改善用の観測ログであり、レビュー通過判断には使わない。

改善ログの分類は次に固定する。

| 分類 | 意味 |
| --- | --- |
| `structure` | 成果物、依存、責務分割、正本配置の問題 |
| `workflow` | 作業流れ、引き継ぎ、終了処理、報告の問題 |
| `permission` | サンドボックス、書き換え範囲、承認、実行権限の問題 |
| `execution` | command、検証、tool、環境実行の問題 |
| `human_feedback` | 人間の修正指示、差し戻し、運用判断 |
| `review_signal` | `reviewback.<観点>.yaml` から作業流れ改善に転用できる示唆 |

改善ログ項目は次の key を持つ。

| key | 意味 |
| --- | --- |
| `event_id` | run 内で安定する識別子 |
| `occurred_at` | 発生時刻または `unknown` |
| `category` | 改善ログの分類 |
| `summary` | 短い事実説明 |
| `evidence_ref` | 根拠の path、command、会話ログ参照 |
| `impact` | `blocker`、`major`、`minor`、`note` のいずれか |
| `next_improvement` | 次回改善へ戻せる具体案 |
| `source` | `implement_lane`、`human`、`reviewback`、`validation`、`work_reporter` のいずれか |

検証証跡 は次をすべて含む。

- 実行コマンド: 呼び出し元が実行した検証コマンド。
- 証跡位置: 実行日時または run 内の証跡位置。
- 成否: pass または fail。
- coverage 値: coverage を測定した場合の値。
- issue 数: security、reliability、maintainability の issue 数。
- system test 件数: system test の実行件数、成功件数、失敗件数。
- 失敗箇所: fail の場合に原因箇所または失敗した検証名。

シナリオ 候補生成器は次の 6 体に固定する。

| agent | 出力ファイル | 観点 |
| --- | --- | --- |
| `scenario_actor_goal_generator` | `scenario-candidates.actor-goal.md` | アクター目的 |
| `scenario_lifecycle_generator` | `scenario-candidates.lifecycle.md` | ライフサイクル |
| `scenario_state_transition_generator` | `scenario-candidates.state-transition.md` | 状態遷移 |
| `scenario_failure_generator` | `scenario-candidates.failure.md` | 異常系 |
| `scenario_external_integration_generator` | `scenario-candidates.external-integration.md` | 外部連携 |
| `scenario_operation_audit_generator` | `scenario-candidates.operation-audit.md` | 運用・監査 |

### UX事前確認規約

`implement_lane` は UI がある task で frontend 実装が完了した後、frontend 人間レビュー前に `ux_review` を起動する。
`ux_review` は `ux-review.yaml` だけを更新し、プロダクトコードとプロダクトテストを変更しない。
`ux-review.yaml` は `docs/exec-plans/active/<task-id>/` に置く。
`implement_lane` は `ux-review.yaml` の `must_fix_open` と `max_level` を読む。
`blocker` と `major` は frontend 人間レビュー前の修正必須問題として扱う。
`minor` と `nit` は人間レビュー時の注意として扱い、単独では人間レビューを止めない。
`ux-review.yaml` の `review_status: stopped` は人間レビュー前の停止として扱う。

### 合意済みfrontend保護規約

`frontend 実装後人間レビュー` が承認された時点で、`implement_lane` は合意済み frontend 保護対象を固定する。
合意済み frontend 保護対象は、後続の `backend 実装`、`統合境界実装`、`シナリオテスト`、`単体テスト` の起動入力へ渡す。

| 保護対象 | 意味 |
| --- | --- |
| 承認済み画面 | 人間レビューで承認された画面、主要区画、主要導線、状態表示 |
| 承認済み表示規則 | 人間レビューで承認された文言、余白、密度、要素サイズ、既存画面との統一条件 |
| 確認済み実画面 | review URL と確認済み `fakeScenario` |
| UX確認結果 | `ux-review.yaml` の未解決なし判定、または人間承認済み残留事項 |
| 変更禁止範囲 | 承認済み frontend touched files と後続 agent が変更してはいけない範囲 |

## 判断規約

- 次の実行判断は 成果物依存表 の未完了 成果物、満たされた `依存対象`、既存 成果物、対象 skill の完了規約で決める。
- 既存 成果物 がある場合は、対象 skill の完了規約を満たすか確認してから後続 成果物 へ進む。
- 起動先 agent の 起動入力 は、対象 skill の入力規約、完了規約、停止規約に合わせて作る。
- `設計差分図` は `diagrammer` を起動して作る。
- `設計差分図` は、予定変更箇所だけの追加・削除差分を示す コンポーネント図 と シーケンス図 に限定する。
- `設計差分図` は、全体構成図、正本図、変更しない箇所の網羅図として作らない。
- `設計差分図` の起動入力には、シナリオ設計、UI設計がある場合の UI設計、予定変更箇所、追加予定箇所、削除予定箇所、禁止範囲、出力先を含める。
- 実装 agent の起動入力には、`backend_implementer`、`frontend_implementer`、`integration_implementer` のどれを起動するかを必ず明示する。
- `backend_implementer` の起動入力には `implement-backend` を必ず明示する。
- `frontend_implementer` の起動入力には `implement-frontend` を必ず明示する。
- `integration_implementer` の起動入力には `implement-integration` を必ず明示する。
- `ux_review` の起動入力には、UX確認対象差分、実装目的、UI根拠、実装結果、実画面確認入力、変更ファイル、作業計画フォルダ、UX確認YAMLパスを含める。
- `UX事前確認` が `must_fix_open: true` または `review_status: stopped` の場合は、frontend 人間レビューへ進めず、`frontend 実装` の再実行入力または人間への返却を固定する。
- `frontend 実装後人間レビュー` が承認済みの場合は、合意済み frontend 保護対象を後続実装の変更禁止範囲として起動入力へ含める。
- 後続実装で画面、部品、文言、style の変更が必要な場合は、実装を続けず `frontend 実装` の再実行入力または人間への返却を固定する。
- `観測ログ追加` の起動入力には、完成済み実装成果物、完成済みテスト成果物、変更ファイル、合意済み frontend 保護対象、作業計画フォルダを含める。
- `観測ログ追加` は `backend 実装`、`frontend 実装`、`統合境界実装`、`シナリオテスト`、`単体テスト` が必要分だけ揃った後、`最終検証` の前に起動する。
- `観測ログ追加` が停止した場合は、`最終検証` へ進めず、人間または該当 実装 agent への戻しを固定する。
- レビュー agent を起動する前に、ゲート判断用 `reviewback.<観点>.yaml` の作業計画フォルダを確定する。
- レビュー agent 起動入力には、最終検証、coverage、issue 数、system test 件数を含む 検証証跡 を明示する。
- レビュー agent の結果は `reviewback.<観点>.yaml` の `must_fix_open`、`max_level`、`review_status` から レビュー集約規約 の優先度で集約する。
- 構造問題、作業流れ問題、権限問題、実行問題、人間フィードバック、レビュー由来の改善示唆を検出した場合は、改善ログへ追記する。
- `review_signal` は `reviewback.<観点>.yaml` のうち、次回の作業流れ改善に転用できる示唆だけを記録する。
- レビュー agent に改善ログを作成または追記させない。
- `blocker`、`critical`、`major` の未解決指摘がある場合は `implementation_action` を `fix` または `rerun_codex_review` にする。
- `minor`、`nit` だけが未解決の場合は `implementation_action` を `report_residual` または `close` にする。
- 5 観点すべてが `review_status: no_issue` または未解決修正必須問題なしの場合だけ `close` を選べる。
- `implementation_action: close` を選ぶ場合は、作業レポート入力を揃えた後に 作業計画フォルダ を `docs/exec-plans/active/<task-id>/` から `docs/exec-plans/completed/<task-id>/` へ移す。
- `scenario-design`、`ui-design`、実装結果、レビュー結果のいずれかに仕様変更または仕様追加が少しでも含まれる場合は、`正本化判断` を必須成果物にする。
- 仕様変更または仕様追加が human 承認済みの恒久仕様である場合は、`詳細仕様正本反映` を必須成果物にする。
- `詳細仕様正本反映` は `docs/detail-specs/` の上位シナリオ単位の正本へ、human 承認済みの恒久仕様だけを反映する。
- `詳細仕様正本反映` の入力は、`scenario-design`、`ui-design`、実装結果、レビュー結果、承認記録のうち正本化判断で承認済みとされた成果物に限定する。
- 起動先 agent には 文脈 を引き継がず、必要情報を 引き継ぎ入力 に明示する。
- 人間介入 が必要な 成果物 は AI だけで完了にしない。
- 恒久修正、構造整理、探索テスト、軽量変更はこの skill で詳細化しない。
- backend、frontend、統合境界 は別 成果物 として扱い、単一の実装成果物に束ねない。
- UI がある task では `frontend 実装` を必須成果物にし、UI がない task では `frontend 実装` を省略できる。
- UI がある task では `UX事前確認` を必須成果物にし、UI がない task では `UX事前確認` を省略できる。
- UI がある task では `frontend 実装後人間レビュー` を必須成果物にし、UI がない task では `frontend 実装後人間レビュー` を省略できる。
- UI がある task の `frontend 実装` は、`backend 実装` より先に起動する。
- UI がある task の `frontend 実装` は、人間レビュー前に fakeAPI を整備し、実画面で確認できる review URL、確認状態、未確認理由を `frontend 実装後人間レビュー` の入力へ含める。
- UI がある task の `frontend 実装` は、backend 実装、統合境界実装、永続化仕様の代替として fakeAPI を扱わない。
- UI がある task の `frontend 実装後人間レビュー` は、`UX事前確認` の通過後に着手する。
- UI がある task の `backend 実装` と `統合境界実装` は、`合意済みfrontend保護` の固定後に着手する。
- `frontend 実装後人間レビュー` が差し戻しまたは追加質問の場合は、後続実装へ進めず、`frontend 実装` の再実行入力または人間への返却を固定する。
- `統合境界実装` は frontend と backend の接続結果を実画面で確認する。
- `観測ログ追加` は実行時にしか確定しない値、実行後に消える中間状態、消えると原因候補を分離できない分岐理由だけを残す。
- `観測ログ追加` はループや大量処理で同種ログを増やさず、件数、分類、集約、代表的な識別子、最初の失敗、最後の失敗を優先する。
- `実装後ブラウザ確認` の確認 URL、起動状態、操作経路、操作期待値、禁止操作、安全条件、証跡出力先は、承認済みシナリオ、UI 要件契約、実装範囲、最終検証観点から `implement_lane` が定義する。
- `browser_confirmation` は `実装後ブラウザ確認` の実行だけを担当し、期待値の妥当性を判断しない。
- `シナリオテスト` と `単体テスト` は別成果物にし、依存対象が揃った後に並列起動できる。
- タスクの終わったサブエージェントを起動したまま残さず，終わったら逐次で閉じること。

## 非対象規約

- 恒久修正、構造整理、探索テスト、軽量変更は詳細化しない。
- シナリオ設計と UI設計の人間レビューは扱わない。
- 起動先 agent の下位 agent 起動は扱わない。
- レビューエージェントに差分コード，レビュー成果物以外の関係ないものを渡さない。ハーネス結果など。
- プロダクトコードとプロダクトテストは変更しない。

## 出力規約

- 人間向け返却: 人間向けには、成果物依存表 の現在 成果物、着手可能 成果物、停止中 成果物、停止理由を短く返す。
- 起動先向け返却: 起動先 agent 向けには、対象 成果物、満たされた `依存対象`、読むファイル、禁止事項、期待する 成果物 を渡す。
- 設計差分図起動入力: `diagrammer` 向けには、図化目的、根拠参照、予定変更箇所、追加予定箇所、削除予定箇所、禁止範囲、対象作業計画フォルダを渡す。
- 設計差分図: 人間設計レビュー向けには、追加・削除差分のコンポーネント図、追加・削除差分のシーケンス図、根拠参照、検証結果、未決事項を返す。
- 実装後ブラウザ確認起動入力: `browser_confirmation` 向けには、確認 URL、起動状態、操作経路、操作期待値、禁止操作、安全条件、証跡出力先を渡す。
- 実装後ブラウザ確認: 操作確認結果、証跡参照、console または network 異常、未確認理由、戻し先を返す。
- 観測ログ追加起動入力: `observability_implementer` 向けには、完成済み実装成果物、完成済みテスト成果物、変更ファイル、合意済み frontend 保護対象、作業計画フォルダを渡す。
- 観測ログ追加: 追加ログ、追加しない理由、禁止ログ確認、変更ファイル、検証未実行理由を返す。
- UX事前確認起動入力: `ux_review` 向けには、UX確認対象差分、実装目的、UI根拠、実装結果、実画面確認入力、変更ファイル、作業計画フォルダ、UX確認YAMLパスを渡す。
- UX事前確認: `ux-review.yaml` の `review_status`、`must_fix_open`、`max_level`、確認済み状態、未確認状態、UX指摘を返す。
- 合意済みfrontend保護: 承認済み画面、承認済み表示規則、確認済み実画面、UX確認結果、変更禁止範囲を返す。
- レビュー起動入力: レビュー agent 向けには、レビュー対象差分、実装目的、承認済み実装範囲、実装結果、検証証跡、変更ファイル、レビューYAMLパスを渡す。
- 改善ログ: `work_history/runs/<run>/workflow-improvement-log.jsonl` へ追記した改善ログ項目を返す。
- 終了処理返却: 終了処理、停止、戻し では、`作業レポート入力` を揃えるための 根拠 と 作業計画フォルダ の移動結果を返す。

## 完了規約

- 新規実装レーンの次 成果物、起動、人間レビュー、引き継ぎ、正本化、停止、戻し を再解釈なしで判断できる。
- シナリオ 候補成果物 が必要な場合は 6 件揃っている。
- `設計差分図` が人間設計レビュー前に揃っている。
- `設計差分図` が予定変更箇所だけの追加・削除差分を示す コンポーネント図 と シーケンス図 を含んでいる。
- UI が関係する場合は、`ui-design.md` が人間設計レビュー前に揃っている。
- UI が関係する場合は、`frontend 実装` が `backend 実装` より先に完了している。
- UI が関係する場合は、人間レビュー前に fakeAPI による実画面確認ができる状態になり、review URL、確認状態、未確認理由が記録されている。
- UI が関係する場合は、`ux-review.yaml` に `review_status`、`must_fix_open`、`max_level`、確認済み状態、未確認状態が記録されている。
- UI が関係する場合は、`UX事前確認` に `blocker` または `major` の未解決問題が残っていない。
- UI が関係する場合は、`frontend 実装後人間レビュー` の承認が記録されている。
- UI が関係する場合は、`合意済みfrontend保護` が固定されている。
- 人間レビュー が必要な場合は承認、差し戻し、追加質問のいずれかが記録されている。
- `統合境界実装` がある場合は、実画面確認結果が 根拠参照 付きで確認されている。
- `backend 実装`、`frontend 実装`、`統合境界実装`、`シナリオテスト`、`単体テスト` 後は `観測ログ追加`、`最終検証`、`実装後ブラウザ確認`、`レビュー通過根拠` が 根拠参照 付きで確認されている。
- `観測ログ追加` は追加ログ、追加しない理由、禁止ログ確認、変更ファイル、検証未実行理由を含んでいる。
- `実装後ブラウザ確認` は確認 URL、操作経路、操作期待値、証跡参照、未確認理由を含んでいる。
- `レビュー通過根拠` は 5 観点の `reviewback.<観点>.yaml` から behavior、security、responsibility_boundary、その他 の優先度で集約され、`implementation_action` が固定されている。
- DAGで必須とされている成果物が全て用意できていること。
- 5 観点すべての `reviewback.<観点>.yaml` に `must_fix_open`、`max_level`、`review_status` が記録されている。
- 仕様変更または仕様追加がある場合は、`正本化判断` の結果が 根拠参照 付きで記録されている。
- human 承認済みの恒久仕様がある場合は、`詳細仕様正本反映` の完了結果または停止理由が 根拠参照 付きで記録されている。
- `backend 実装` またはテスト変更に backend 変更が含まれる場合は `python3 scripts/harness/run.py --suite backend-local` を `.codex/rules/default.rules` の許可対象として実行し、失敗時は担当 agent がその場で直して再実行した通過結果または未実行理由が確認されている。
- `frontend 実装` またはテスト変更に frontend 変更が含まれる場合は `python3 scripts/harness/run.py --suite frontend-local` を `.codex/rules/default.rules` の許可対象として実行し、失敗時は担当 agent がその場で直して再実行した通過結果または未実行理由が確認されている。
- レビュー agent 起動前に、実行コマンド、証跡位置、成否、coverage 値、issue 数、system test 件数、失敗箇所を含む 検証証跡 が揃っている。
- `workflow-improvement-log.jsonl` が必要な場合は、分類、根拠、次回改善が JSONL として追記されている。
- 終了処理、停止、戻し のいずれでも `作業レポート入力` と 作業観測根拠 が作成されている。
- `implementation_action: close` の場合は、作業計画フォルダ が `docs/exec-plans/completed/<task-id>/` に移動済みで、`docs/exec-plans/active/<task-id>/` に残っていない。

## 停止規約

- 依頼が新規実装または機能拡張か判断できない場合は停止する。
- `designer`、`investigator` の必要判定ができない場合は停止する。
- `設計差分図` なしで `人間設計レビュー` へ進みそうな場合は停止する。
- `設計差分図` が予定変更箇所以外を網羅図として含む場合は停止する。
- 人間レビュー が必要な判断を AI だけで確定しそうな場合は停止する。
- 承認済み `実装範囲` なしで `backend 実装`、`frontend 実装`、`統合境界実装` が必要な場合は停止する。
- UI が関係する task で fakeAPI による実画面確認の review URL、確認状態、未確認理由が不足するまま `frontend 実装後人間レビュー` へ進みそうな場合は停止する。
- UI が関係する task で `UX事前確認` が不足するまま `frontend 実装後人間レビュー` へ進みそうな場合は停止する。
- UI が関係する task で `UX事前確認` に `blocker` または `major` の未解決問題が残るまま `frontend 実装後人間レビュー` へ進みそうな場合は停止する。
- UI が関係する task で `frontend 実装後人間レビュー` の承認がないまま `合意済みfrontend保護` へ進みそうな場合は停止する。
- UI が関係する task で `合意済みfrontend保護` がないまま `backend 実装`、`統合境界実装`、`最終検証` へ進みそうな場合は停止する。
- `観測ログ追加` なしで `最終検証` へ進みそうな場合は停止する。
- `観測ログ追加` の停止理由が未解決のまま `最終検証` へ進みそうな場合は停止する。
- `実装後ブラウザ確認` の確認 URL、起動状態、操作経路、操作期待値、禁止操作、安全条件、証跡出力先が不足する場合は停止する。
- `実装後ブラウザ確認` なしで `レビュー通過根拠` へ進みそうな場合は停止する。
- `python3 scripts/harness/run.py --suite all` の失敗原因が承認済み実装範囲 外にある場合は停止する。
- レビュー agent 起動入力に 検証証跡 が不足する場合は停止する。
- 最終検証 または `レビュー通過根拠` が不明なまま正本化が必要な場合は停止する。
- 仕様変更または仕様追加があるのに `正本化判断` が不足する場合は終了不可とする。
- human 承認済みの恒久仕様があるのに `詳細仕様正本反映` が不足する場合は終了不可とする。
- `implementation_action: close` の状態で 作業計画フォルダ を `docs/exec-plans/completed/<task-id>/` へ移動できない場合は終了不可とする。
- `作業レポート入力` または 作業観測根拠 が不足する場合は終了不可とする。
