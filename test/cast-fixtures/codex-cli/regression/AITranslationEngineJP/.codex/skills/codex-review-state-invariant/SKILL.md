---
name: codex-review-state-invariant
description: Codex 実装後 レビュー の状態・データ不変条件グループ作業プロトコル。
---
# Codex Review State Invariant

## 目的

DB、キャッシュ、非同期処理、再実行、同時実行で壊れないかを見る。
本番で壊れやすい状態遷移を、diff から取得した実コードで採点する。

## 対応ロール

- `review_state_invariant` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` の レビュー 集約 とする。
- 担当成果物は `codex-review-state-invariant` の出力規約で固定する。

## 入力規約

- レビュー対象差分: 実装後 レビュー の対象になる差分を受け取る。
- 実装目的: レビュー対象差分が満たすべき目的を受け取る。
- implementation-scope の場所: 承認済み実装範囲の参照先を受け取る。
- 実装結果: 実装 agent が返した実装結果を受け取る。
- 検証証跡: 実行コマンド、証跡位置、成否、coverage 値、issue 数、system test 件数、失敗箇所を受け取る。
- 変更ファイル: レビュー対象差分に含まれる変更ファイル一覧を受け取る。
- 作業計画フォルダ: `docs/exec-plans/active/<task-id>/` を受け取る。
- レビューYAMLパス: `docs/exec-plans/active/<task-id>/reviewback.state-invariant.yaml` を受け取る。

## 外部参照規約

- エージェント実行定義と実行境界は [review_state_invariant.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/review_state_invariant.toml) に従う。
- レビューYAMLの正本形式は [reviewback.yaml](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/exec-plans/templates/task-folder/reviewback.yaml) に従う。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

状態・データ不変条件観点表は次を拘束する。

| 観点 | 確認内容 |
| --- | --- |
| transaction / lock | 複数更新、排他、失敗時 rollback で整合性が崩れないか |
| retry / 冪等性 | 再実行、二重送信、途中失敗で重複副作用が起きないか |
| 競合状態 | 同時実行、DB 更新順序、読み取り後更新で不整合が起きないか |
| cache / event | キャッシュ無効化、イベント発行、購読側反映で古い状態が残らないか |
| queue / 部分失敗 | consumer 失敗、再配送、途中成功で処理済み境界が壊れないか |
| データ整合性 | 論理削除、集計値、二重作成、二重課金が発生しないか |

重大度指標は次を拘束する。

| レベル | 意味 |
| --- | --- |
| `blocker` | マージ・リリースを止めるべき問題 |
| `critical` | 修正必須だが、条件付きで先に進める可能性がある問題 |
| `major` | 品質・保守性・仕様整合に大きく影響する問題 |
| `minor` | 局所的な改善で済む問題 |
| `nit` | 修正してもよいが、必須ではない問題 |

## 判断規約

- レビュー問題の重大度は内部参照規約の重大度指標から選ぶ。
- `blocker`、`critical`、`major` は `fix_required: true` にする。
- `minor`、`nit` は `fix_required: false` にする。
- 未解決の `fix_required: true` がある場合は `must_fix_open: true` にする。
- `max_level` は未解決指摘の最大重大度にする。
- 再実行不能、部分失敗、二重処理の可能性は高い重大度にする。
- 呼び出し元から渡された検証証跡をレビュー入力として扱ってよい。

## 非対象規約

- SQL の見た目、命名、UI 文言、内部設計の美しさは主判定にしない。
- テストコードの構成は、状態・データ不変条件の直接根拠になる場合だけ扱う。
- ハーネスを実行しない。
- 修正範囲の命令やプロダクトコード変更の指示は出力しない。

## 出力規約

- レビューYAML: `docs/exec-plans/active/<task-id>/reviewback.state-invariant.yaml` を作成、追記、解決更新、削除する。
- レビューYAML形式: [reviewback.yaml](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/exec-plans/templates/task-folder/reviewback.yaml) の項目、説明、記入条件に従う。
- レビューYAML観点: `viewpoint` は `state-invariant`、`reviewer_agent` は `review_state_invariant` とする。
- 改善ログ: 作成または追記しない。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示、修正範囲の命令を含めない。

## 完了規約

- `reviewback.state-invariant.yaml` に 対象 レビュー 観点の指摘、不変条件維持度、根拠、残留リスクが記録されている。
- 内部参照規約の状態・データ不変条件観点表を確認した。
- 検証証跡の実行コマンド、証跡位置、成否、coverage 値、issue 数、system test 件数、失敗箇所を確認した。
- 破られた不変条件と原因候補を分けた。
- 局所修正評価と不変条件テスト観点を返した。
- 完了判断材料として、`must_fix_open`、`max_level`、不変条件維持度、破られた不変条件、原因候補、局所修正評価、根拠が記録されている。
- 残留リスクとして、未確認範囲と理由が記録されている。

## 停止規約

- `レビュー対象差分` が不足する場合は停止する。
- `実装目的` が不足する場合は停止する。
- `検証証跡` が不足する場合は停止する。
- `レビューYAMLパス` が不足する場合は停止する。
- 外部成果物 が不足または衝突する場合は停止する。
- 状態・データ不変条件以外の観点を主判定にしそうな場合は停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
