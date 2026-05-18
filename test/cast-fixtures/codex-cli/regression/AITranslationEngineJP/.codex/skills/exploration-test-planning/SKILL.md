---
name: exploration-test-planning
description: 探索テストレーンの探索計画を固定する作業プロトコル。
---
# Exploration Test Planning

## 目的

`exploration-test-planning` は、探索テストの観測対象、探索観点、テストデータ方針、停止条件を固定する作業プロトコルである。
`exploration_test_planner` が探索証跡の前提になる計画成果物を作る時に使う。

## 対応ロール

- `exploration_test_planner` が使う。
- 呼び出し元は `exploration_test_lane` とする。
- 返却先は `exploration_test_lane` とする。
- 担当成果物は `探索計画` とする。

## 入力規約

- 呼び出し元: この skill を呼び出した agent。
- 依頼要約: 探索テストで確認する依頼内容。
- 作業計画フォルダ: task 内成果物を置く `docs/exec-plans/active/<task-id>/`。
- 既存成果物: 作業計画フォルダに既にある task 内成果物。
- 非必須入力: 人間が指定した探索対象または除外対象。

## 外部参照規約

- エージェント実行定義と実行境界は [exploration_test_planner.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/exploration_test_planner.toml) に従う。
- 探索テストレーンの成果物DAGは [README.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/README.md) に従う。
- 探索計画の雛形は [exploration-test-plan.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/exploration-test-planning/assets/exploration-test-plan.md) とする。
- 探索計画の task 内 artifact は `docs/exec-plans/active/<task-id>/exploration-test-plan.md` とする。
- 外部成果物が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

探索計画は次の項目を持つ。

| 項目 | 意味 |
| --- | --- |
| `観測対象` | 探索証跡で確認する画面、操作、API、状態、ログの範囲 |
| `探索観点` | 探索証跡で確認する失敗、状態遷移、復旧、ログ、権限などの観点 |
| `テストデータ方針` | テストデータで用意する入力値、前提状態、再利用する fixture の方針 |
| `停止条件` | 証跡が十分な状態、再現不能、環境停止、人間判断待ちの条件 |
| `planning_blockers` | 探索計画を完了するまでに修正が必要な不足、衝突、曖昧さ |
| `correction_history` | `planning_blockers` ごとの修正内容と残り不足 |

## 判断規約

- 観測、ログ確認、画面確認、原因仮説の作成を行わない。
- 探索証跡で確認する範囲を、観測対象と探索観点として固定する。
- テストデータの具体値を作らず、テストデータ方針だけを固定する。
- 探索範囲が広すぎる場合は、探索証跡を再解釈なしで作れる粒度へ分ける。
- 探索計画の不足、衝突、曖昧さは停止せず、`planning_blockers` として返す。
- `planning_blockers` を受けた再起動では、修正結果を `correction_history` に追記する。
- `planning_blockers` が空になるまで、判断結果を完了にしない。

## 非対象規約

- 探索証跡は扱わない。
- バグ一覧、ログ、影響ファイルの集約は扱わない。
- プロダクトコードとプロダクトテストは変更しない。
- 恒久修正と回帰テスト実装は扱わない。

## 出力規約

- 判断結果: 探索計画の完了、未完了、停止の判定を返す。
- 根拠参照: 計画判断に使った依頼、既存成果物、除外対象を返す。
- 不足情報: 探索計画を完了できない不足項目を返す。
- 次判断材料: `exploration_test_lane` がテストデータと探索証跡を判断できる材料を返す。
- 観測対象: 探索証跡で確認する対象範囲を返す。
- 探索観点: 探索証跡で確認する観点を返す。
- テストデータ方針: テストデータの用意方針を返す。
- 停止条件: 探索証跡を止める条件を返す。
- 計画 blocker: 探索計画を完了するまでに修正が必要な不足、衝突、曖昧さを返す。
- 修正履歴: `planning_blockers` ごとの修正内容と残り不足を返す。
- 禁止事項: 出力に観測結果、原因仮説、プロダクトコード変更の指示を含めない。

## 完了規約

- 観測対象、探索観点、テストデータ方針、停止条件が返っている。
- `exploration-test-plan.md` に探索計画が記録されている。
- `investigator` が探索証跡を作れる粒度で探索範囲が固定されている。
- `exploration_test_lane` がテストデータを判断できる。
- `planning_blockers` が空である。
- `planning_blockers` の修正履歴がある場合は `correction_history` に記録されている。

## 停止規約

- 依頼内容から探索対象を決められない場合は停止する。
- 観測対象、探索観点、テストデータ方針、停止条件の不足は停止せず、`planning_blockers` として返す。
- プロダクトコードまたはプロダクトテストの変更が必要な場合は停止する。
- 人間判断が必要な探索範囲を AI だけで確定しそうな場合は停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
