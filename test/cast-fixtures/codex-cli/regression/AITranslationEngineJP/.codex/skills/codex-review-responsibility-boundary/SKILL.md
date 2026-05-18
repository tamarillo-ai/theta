---
name: codex-review-responsibility-boundary
description: Codex 実装後 レビュー の責務境界グループ作業プロトコル。
---
# Codex Review Responsibility Boundary

## 目的

変更後のコードが、層責務、依存方向、関数責務を越えていないかを見る。
差分から取得した実コードを、承認済み実装範囲とアーキテクチャ正本に照らして採点する。

## 対応ロール

- `review_responsibility_boundary` が使う。
- 呼び出し元は `implement_lane` または `light_change_lane` とする。
- 返却先は呼び出し元の レビュー 集約 とする。
- 担当成果物は `codex-review-responsibility-boundary` の出力規約で固定する。

## 入力規約

- レビュー対象差分: 実装後 レビュー の対象になる差分を受け取る。
- 実装目的: レビュー対象差分が満たすべき目的を受け取る。
- 承認済み範囲の場所: `implementation-scope` または 軽量変更レーンの `task 枠` の参照先を受け取る。
- 実装結果: 実装 agent が返した実装結果を受け取る。
- 検証証跡: 実行コマンド、証跡位置、成否、coverage 値、issue 数、system test 件数、失敗箇所を受け取る。
- 変更ファイル: レビュー対象差分に含まれる変更ファイル一覧を受け取る。
- 作業計画フォルダ: `docs/exec-plans/active/<task-id>/` を受け取る。
- レビューYAMLパス: `docs/exec-plans/active/<task-id>/reviewback.responsibility-boundary.yaml` を受け取る。

## 外部参照規約

- 構造責務の正本は [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) とする。
- 実装規約の入口は [coding-guidelines.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/coding-guidelines.md) とする。
- エージェント実行定義と実行境界は [review_responsibility_boundary.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/review_responsibility_boundary.toml) に従う。
- レビューYAMLの正本形式は [reviewback.yaml](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/exec-plans/templates/task-folder/reviewback.yaml) に従う。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

責務境界観点表は次を拘束する。

| 観点 | 確認内容 |
| --- | --- |
| 層責務 | View、ScreenController、UseCase、Service、Repository、アダプター が正本上の責務を越えていないか |
| 依存方向 | Bootstrap 以外の層が具象実装を生成していないか |
| 関数責務 | 変換、検証、永続化、副作用、表示整形が 1 関数へ混在していないか |
| 境界変換 | 公開接点の値、内部値、画面表示用値の変換位置が正しいか |
| 副作用配置 | ファイルシステム、実行時基盤、データベース接続、提供元固有処理が中核側へ漏れていないか |
| 変更範囲 | 承認済み実装範囲を越えた構造変更や横断整理が混ざっていないか |

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
- 構造責務の正本に反する依存方向、層越境、複数責務の混在は高い重大度にする。
- 自動検査で検出済みの境界違反だけに限定せず、実コード上の責務混在を判断対象にする。
- 呼び出し元から渡された検証証跡をレビュー入力として扱ってよい。

## 非対象規約

- 挙動正しさ、契約・互換性、権限・信頼境界、状態・データ不変条件は主判定にしない。
- 命名、好みの分割、見た目だけの読みやすさは主判定にしない。
- 修正範囲の命令やプロダクトコード変更の指示は出力しない。
- ハーネスを実行しない。

## 出力規約

- レビューYAML: `docs/exec-plans/active/<task-id>/reviewback.responsibility-boundary.yaml` を作成、追記、解決更新、削除する。
- レビューYAML形式: [reviewback.yaml](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/exec-plans/templates/task-folder/reviewback.yaml) の項目、説明、記入条件に従う。
- レビューYAML観点: `viewpoint` は `responsibility-boundary`、`reviewer_agent` は `review_responsibility_boundary` とする。
- 改善ログ: 作成または追記しない。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示、修正範囲の命令を含めない。

## 完了規約

- `reviewback.responsibility-boundary.yaml` に 対象 レビュー 観点の指摘、責務境界維持度、根拠、残留リスクが記録されている。
- 内部参照規約の責務境界観点表を確認した。
- 検証証跡の実行コマンド、証跡位置、成否、coverage 値、issue 数、system test 件数、失敗箇所を確認した。
- 破られた境界と原因候補を分けた。
- 局所修正評価と境界固定テスト観点を返した。
- 挙動正しさ、契約・互換性、権限・信頼境界、状態・データ不変条件を主判定にしなかった。
- 完了判断材料として、`must_fix_open`、`max_level`、責務境界維持度、破られた境界、原因候補、局所修正評価、根拠が記録されている。
- 残留リスクとして、未確認範囲と理由が記録されている。

## 停止規約

- `レビュー対象差分` が不足する場合は停止する。
- `実装目的` が不足する場合は停止する。
- `検証証跡` が不足する場合は停止する。
- `レビューYAMLパス` が不足する場合は停止する。
- 外部成果物 が不足または衝突する場合は停止する。
- 責務境界以外の観点を主判定にしそうな場合は停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
