---
name: exploration-test-lane
description: 探索テストレーンの成果物DAG、起動入力、集約、停止、終了条件を固定する作業プロトコル。
---
# Exploration Test Lane

## 目的

`exploration-test-lane` は、探索テストの進行判断を task 内成果物DAG と 引き継ぎへ固定する作業プロトコルである。
`exploration_test_lane` が探索計画、テストデータ、探索証跡、バグ一覧、実装証跡、回帰テスト証跡を管理する時に使う。

## 対応ロール

- `exploration_test_lane` が使う。
- 呼び出し元は人間とする。
- 返却先は人間とする。
- 担当成果物は `テストデータ`、`バグ一覧とログ、影響ファイル`、`作業レポート入力` とする。

## 入力規約

- 呼び出し元: この skill を呼び出した人間または戻し元。
- 依頼要約: 探索テストとして扱う依頼内容。
- 作業計画フォルダ: task 内成果物を置く `docs/exec-plans/active/<task-id>/`。
- 既存成果物: 作業計画フォルダに既にある task 内成果物。
- 人間介入状態: 人間レビュー、承認、差し戻し、追加質問の記録。

## 外部参照規約

- エージェント実行定義と実行境界は [exploration_test_lane.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/exploration_test_lane.toml) に従う。
- 探索計画は [exploration-test-planning](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/exploration-test-planning/SKILL.md) に従う。
- 探索証跡は [investigate](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/investigate/SKILL.md) に従う。
- プロダクトコード実装は `implement-backend`、`implement-frontend`、`implement-integration` のいずれかに従う。
- 回帰テスト証跡は `tests-scenario` または `tests-unit` に従う。
- テストデータの雛形は [exploration-test-data.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/exploration-test-lane/assets/exploration-test-data.md) とする。
- バグ一覧、ログ、影響ファイルの雛形は [exploration-test-findings.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/exploration-test-lane/assets/exploration-test-findings.md) とする。
- 回帰テスト証跡の雛形は [regression-test-evidence.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/exploration-test-lane/assets/regression-test-evidence.md) とする。
- 探索テストの task 内 artifact は `exploration-test-plan.md`、`exploration-test-data.md`、`exploration-test-evidence.md`、`exploration-test-findings.md`、`regression-test-evidence.md` とする。
- 外部成果物が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

探索テストレーンの成果物DAGは次を必ず持つ。
各成果物は、`依存対象` の成果物が揃った時だけ着手できる。

| 成果物ID | 担当者 | 依存対象 | 次 agent |
| --- | --- | --- | --- |
| `探索計画` | `exploration_test_planner` | `task 枠` | `exploration_test_planner` |
| `テストデータ` | `exploration_test_lane` | `探索計画` | なし |
| `探索証跡` | `investigator` | `探索計画`, `テストデータ` | `investigator` |
| `バグ一覧とログ、影響ファイル` | `exploration_test_lane` | `探索証跡` | なし |
| `実装証跡` | 実装種別別 agent | `バグ一覧とログ、影響ファイル` | `backend_implementer` または `frontend_implementer` または `integration_implementer` |
| `回帰テスト証跡` | `implementation_scenario_tester` または `implementation_unit_tester` | `実装証跡` | `implementation_scenario_tester` または `implementation_unit_tester` |
| `レビュー通過根拠` | `exploration_test_lane` | `探索計画`, `探索証跡`, `バグ一覧とログ、影響ファイル`, `実装証跡?`, `回帰テスト証跡?` | `review_behavior`, `review_contract`, `review_trust_boundary`, `review_state_invariant`, `review_responsibility_boundary` |
| `作業レポート入力` | `exploration_test_lane` / `work_reporter` | 全完了または停止済み成果物, `レビュー通過根拠?` | `work_reporter` |

## 判断規約

- 次の実行判断は成果物DAGの未完了成果物、満たされた `依存対象`、既存成果物、対象 skill の完了規約で決める。
- 起動先 agent の起動入力には、対象成果物、満たされた `依存対象`、読むファイル、禁止事項、期待する成果物を明示する。
- `探索計画` は `exploration_test_planner` へ渡す。
- `探索計画` に `planning_blockers` がある場合は停止せず、`planning_blockers` と `correction_history` を渡して `exploration_test_planner` を再起動する。
- `探索計画` は `planning_blockers` が空になるまで後続成果物へ進めない。
- `探索証跡` は `investigator` へ渡す。
- `バグ一覧とログ、影響ファイル` は探索証跡の観測事実、UI 証跡、ログ、未確認事項から集約する。
- `実装証跡` は実装種別別 agent へ渡し、実装 agent と実装 skill を `backend_implementer` / `implement-backend`、`frontend_implementer` / `implement-frontend`、`integration_implementer` / `implement-integration` のいずれか 1 組に固定する。
- `回帰テスト証跡` は変更範囲と検証目的から `implementation_scenario_tester` または `implementation_unit_tester` へ渡す。
- `レビュー通過根拠` は探索計画、探索証跡、バグ一覧、実装証跡、回帰テスト証跡を入力にして観点別レビュー agent を起動する。
- 観点別レビュー agent の結果は `reviewback.<観点>.yaml` に記録する。
- プロダクトコードとプロダクトテストは変更しない。

## 非対象規約

- 探索計画の作成は扱わない。
- 探索証跡の観測は扱わない。
- プロダクトコード実装は扱わない。
- プロダクトテスト実装は扱わない。
- docs 正本化本文の更新は扱わない。

## 出力規約

- 人間向け返却: 成果物DAGの現在成果物、着手可能成果物、停止中成果物、停止理由を返す。
- 起動先向け返却: 起動先 agent 向けに対象成果物、満たされた `依存対象`、読むファイル、禁止事項、期待する成果物を返す。
- テストデータ: 探索計画に対応する入力値、前提状態、再利用 fixture を返す。
- バグ一覧: 探索証跡から確認したバグ候補、再現条件、ログ参照、影響ファイルを返す。
- 探索テスト artifact: 更新した探索テスト artifact の path を返す。
- 計画修正履歴: `planning_blockers` と修正結果を返す。
- レビュー起動入力: レビュー agent 向けに探索計画、探索証跡、バグ一覧、実装証跡、回帰テスト証跡、レビューYAMLパスを返す。
- 作業レポート入力: 完了または停止した成果物、検証、残留リスク、次に見るべき場所を返す。
- 禁止事項: 出力にプロダクトコード、プロダクトテスト、docs 正本本文の変更を含めない。

## 完了規約

- 探索テストレーンの次成果物、起動、停止、戻しを再解釈なしで判断できる。
- 探索計画、テストデータ、探索証跡、バグ一覧とログ、影響ファイルが根拠参照付きで確認されている。
- 探索計画の `planning_blockers` が空であり、修正履歴がある場合は根拠参照付きで確認されている。
- `exploration-test-data.md`、`exploration-test-findings.md`、必要な場合は `regression-test-evidence.md` が作業計画フォルダに記録されている。
- 実装証跡が必要な場合は、実装担当 agent の完了結果が確認されている。
- 回帰テスト証跡が必要な場合は、test agent の完了結果が確認されている。
- 5 観点の `reviewback.<観点>.yaml` が確認されている。
- 終了処理、停止、戻しのいずれでも `作業レポート入力` と 作業観測根拠が作成されている。

## 停止規約

- 依頼が探索テストか判断できない場合は停止する。
- 探索計画なしでテストデータまたは探索証跡へ進みそうな場合は停止する。
- `planning_blockers` が残ったまま後続成果物へ進みそうな場合は停止する。
- 探索証跡なしでバグ一覧、ログ、影響ファイルを集約しそうな場合は停止する。
- 実装 skill を 1 つに固定できない場合は停止する。
- 回帰テスト証跡の担当 agent を決められない場合は停止する。
- プロダクトコードまたはプロダクトテストを直接変更しそうな場合は停止する。
- レビュー agent 起動入力に探索計画、探索証跡、バグ一覧、実装証跡、回帰テスト証跡の必要分が不足する場合は停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
