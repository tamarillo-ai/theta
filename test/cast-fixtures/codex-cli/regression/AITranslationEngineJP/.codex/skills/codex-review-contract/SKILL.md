---
name: codex-review-contract
description: Codex 実装後 レビュー の契約・互換性グループ作業プロトコル。
---
# Codex Review Contract

## 目的

既存利用者、外部 API、内部 API、DB schema、event 入力内容 を壊していないかを見る。
コード自体が動いても契約破壊が利用者側障害になるため、diff の public 境界 を採点する。

## 対応ロール

- `review_contract` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` の レビュー 集約 とする。
- 担当成果物は `codex-review-contract` の出力規約で固定する。

## 入力規約

- レビュー対象差分: 実装後 レビュー の対象になる差分を受け取る。
- 実装目的: レビュー対象差分が満たすべき目的を受け取る。
- implementation-scope の場所: 承認済み実装範囲の参照先を受け取る。
- 実装結果: 実装 agent が返した実装結果を受け取る。
- 最終検証結果: `implement_lane` が確認した最終検証結果を受け取る。
- 検証証跡: 実行コマンド、証跡位置、成否、coverage 値、issue 数、system test 件数、失敗箇所を受け取る。
- 変更ファイル: レビュー対象差分に含まれる変更ファイル一覧を受け取る。
- 作業計画フォルダ: `docs/exec-plans/active/<task-id>/` を受け取る。
- レビューYAMLパス: `docs/exec-plans/active/<task-id>/reviewback.contract.yaml` を受け取る。

## 外部参照規約

- エージェント実行定義と実行境界は [review_contract.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/review_contract.toml) に従う。
- レビューYAMLの正本形式は [reviewback.yaml](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/exec-plans/templates/task-folder/reviewback.yaml) に従う。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

契約・互換性観点表は次を拘束する。

| 観点 | 確認内容 |
| --- | --- |
| API request / response | 既存の入力、出力、エラー応答の意味を壊していないか |
| DB schema | 既存データ、制約、移行前後の読み書き互換性を壊していないか |
| public method | 呼び出し側が依存する引数、戻り値、エラー条件を変えていないか |
| event 入力内容 | 発行内容、購読側の期待、順序、欠落時の扱いを壊していないか |
| エラー code | 既存利用者が分岐に使う code を不要に変えていないか |
| nullable / 必須 | 既存 field の null 許容と必須性を破っていないか |
| versioning | 互換性を壊す変更に版管理または移行経路があるか |

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
- 既存 field の意味変更や nullable / 必須 の変更は高い重大度にする。
- 呼び出し元から渡された検証証跡をレビュー入力として扱ってよい。
- 広い ハーネス 再実行を レビュー agent の責務にしない。

## 非対象規約

- 内部実装の綺麗さ、可読性、パフォーマンス最適化は主判定にしない。
- テストの十分性は、契約・互換性の直接根拠になる場合だけ扱う。
- 広い ハーネス 再実行は扱わない。
- 修正範囲の命令やプロダクトコード変更の指示は出力しない。

## 出力規約

- レビューYAML: `docs/exec-plans/active/<task-id>/reviewback.contract.yaml` を作成、追記、解決更新、削除する。
- レビューYAML形式: [reviewback.yaml](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/exec-plans/templates/task-folder/reviewback.yaml) の項目、説明、記入条件に従う。
- レビューYAML観点: `viewpoint` は `contract`、`reviewer_agent` は `review_contract` とする。
- 改善ログ: 作成または追記しない。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示、修正範囲の命令を含めない。

## 完了規約

- `reviewback.contract.yaml` に 対象 レビュー 観点の指摘、互換性評価、根拠、残留リスクが記録されている。
- 内部参照規約の契約・互換性観点表を確認した。
- 検証証跡の実行コマンド、証跡位置、成否、coverage 値、issue 数、system test 件数、失敗箇所を確認した。
- 破られた不変条件と原因候補を分けた。
- 局所修正評価と不変条件テスト観点を返した。
- 内部実装の綺麗さを主判定にしなかった。
- 完了判断材料として、`must_fix_open`、`max_level`、互換性評価、破られた不変条件、原因候補、局所修正評価、根拠が記録されている。
- 残留リスクとして、未確認範囲と理由が記録されている。

## 停止規約

- `レビュー対象差分` が不足する場合は停止する。
- `実装目的` が不足する場合は停止する。
- `検証証跡` が不足する場合は停止する。
- `レビューYAMLパス` が不足する場合は停止する。
- 外部成果物 が不足または衝突する場合は停止する。
- 契約・互換性以外の観点を主判定にしそうな場合は停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
