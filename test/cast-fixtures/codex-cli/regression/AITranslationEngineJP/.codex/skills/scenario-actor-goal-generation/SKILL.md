---
name: scenario-actor-goal-generation
description: Codex 側の actor-goal シナリオ 候補生成 skill。アクターの目的、開始操作、成功体験から シナリオ 候補を作る。
---
# アクター目的シナリオ候補生成

## 目的

`scenario-actor-goal-generation` は作業プロトコルである。
`scenario_actor_goal_generator` が actor-goal 観点 の シナリオ 候補だけを作る時に使う。

出力形、完了条件、停止条件はこの skill に従う。

## 対応ロール

- `scenario_actor_goal_generator` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `designer` とする。
- 担当成果物は `scenario-actor-goal-generation` の出力規約で固定する。

## 入力規約

- 実行中タスク成果物場所: 候補成果物を読み書きする作業計画フォルダまたは run 成果物フォルダ。
- 対象差分: シナリオ候補生成の対象にする変更差分。

## 外部参照規約

- エージェント実行定義と実行境界は [scenario_actor_goal_generator.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/scenario_actor_goal_generator.toml) に従う。
- 要件正本: [spec.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/spec.md) とする。
- architecture 正本: [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) とする。
- ER 正本: [er.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/er.md) と [diagrams/er](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/diagrams/er/) とする。
- 画面正本: [screen-design](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/screen-design/README.md) とする。
- 上位シナリオ詳細仕様正本: [detail-specs](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/detail-specs/README.md) とする。
- scenario 正本: [scenario-tests](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/scenario-tests/README.md) とする。
- 候補成果物雛形: [scenario-candidates.viewpoint.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-actor-goal-generation/assets/scenario-candidates.viewpoint.md) とする。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。
- 統合先規約: [scenario-design](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/scenario-design/SKILL.md) を参照する。

## 内部参照規約

### 観点

| 観点 | 説明 |
| --- | --- |
| 達成目的 | 人間、外部システム、内部処理が達成したい結果を候補の起点にする。 |
| 開始経路 | UI 操作、API 呼び出し、後続作業開始を混ぜず、候補ごとに 1 つの開始経路へ分ける。 |
| 成功結果 | 主要な正常系と代替成功を分け、実行者にとって成功したと判断できる結果を書く。 |
| 観測点 | 実行者の成功判定を、画面表示、API 応答、永続化状態、履歴のいずれかで確認できる点へつなげる。 |
| 人間判断候補 | 実行者、目的、成功判定が外部正本と対象差分だけで決められない場合に残す。 |

## 判断規約

- 判断は入力 成果物、外部参照規約、対象 agent の責務境界に従う。

## 非対象規約

- 状態遷移網羅と外部連携失敗を主目的にしない。
- 最終シナリオ表の確定、候補の採否、統合判断は扱わない。
- プロダクト実装、未承認 docs 正本化、ツール権限、プロダクト仕様正本は扱わない。

## 出力規約

- 観点: `actor-goal` 観点であることを返す。
- 成果物: `docs/exec-plans/active/<task-id>/scenario-candidates.actor-goal.md` を返す。
- 候補: 実行者、目的、開始条件、期待結果、観測点を持つ候補を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。
- 候補数: 生成した 候補 シナリオ 数を返す。0 件なら不足理由を返す。
- 根拠網羅: 候補 ごとの 根拠要件、関連する詳細要求タイプ、観測点を返す。
- 競合候補: 他 観点 や最終 シナリオ 統合時に競合しうる前提、状態、結果、検証段階を返す。
- 人間判断 候補: AI が確定できない業務判断、状態遷移、外部連携、監査保存対象を返す。

## 完了規約

- 指定 観点 の 候補成果物 が出力規約の必須項目を満たしている。
- 採否や統合判断を行わず、designer が判断できる候補として返却されている。
- 必須 根拠: 実行中タスク成果物場所、対象差分、候補成果物パス、観点を返している。
- 完了判断材料: implement_lane が designer 起動入力に入れる 候補成果物パス、候補数、競合候補、人間判断候補 を判断できる。
- 残留リスク: AI が確定できない判断候補が返っている。

## 停止規約

- 停止時は不足項目、衝突箇所、戻し先を返す。
- 最終シナリオ表 の確定が求められている場合は停止する。
- 候補 採否または統合判断が求められている場合は停止する。
- プロダクト実装が求められている場合は停止する。
- 未承認 docs 正本化が求められている場合は停止する。
- 実行中タスク成果物場所が不足している場合は停止する。
- 対象差分が不足している場合は停止する。
- 候補成果物 の書き先が 実行中タスク成果物場所 外である場合は停止する。
- 人間レビュー が必要な判断を AI だけで確定しそうな場合は停止する。
