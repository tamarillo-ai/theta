---
name: implementation-investigate-reobserve
description: Codex implementation レーン 側の修正後再観測作業プロトコル。
---
# Implementation Investigate Reobserve

## 目的

この skill は作業プロトコルである。
`implementation_investigator` agent が実装後または test 後に同じ条件で再観測する時の判断基準を提供する。

## 対応ロール

- `implementation_investigator` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` とする。
- 担当成果物は `implementation-investigate-reobserve` の出力規約で固定する。

## 入力規約

- 単一調査入力: 実装時調査用 引き継ぎから切り出された再観測作業 1 件。
- 実行中タスク成果物場所: 再観測結果、停止理由を書き戻す作業計画フォルダまたは run 成果物フォルダ。
- 再観測対象: 修正後に再確認する操作、コマンド、入力、公開接点。
- 対象調査範囲: 再観測のために読んでよいファイル、symbol、ログ、成果物。
- 検証コマンド: 再観測結果を確認する実行許可済み command。

## 外部参照規約

- エージェント実行定義と実行境界は [implementation_investigator.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/implementation_investigator.toml) に従う。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

## 判断規約

- 事前の reproduction condition と同じ条件で観測する
- 変更前後の差を 観測済み事実 として返す
- 未解消ケースを remaining_gaps に残す
- 実装修正を同時に行わない

- コマンド、入力、期待、実際を比較する
- 残留リスク を根拠付きで残す
- recommended_next_step を返す

## 非対象規約

- 初回再現、実装中 trace、レビュー判定だけの作業は扱わない。
- 条件を変えて通過扱いにしない。
- 実装修正とプロダクトテスト追加は扱わない。

## 出力規約

- 判断結果: 再観測作業の完了、未完了、停止の判定を返す。
- 根拠参照: 再観測の根拠にした入力、コマンド、観測結果を返す。
- 不足情報: 再観測を完了できない不足項目を返す。
- 次判断材料: 次 agent が判断できる材料を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコードの変更義務を含めない。

## 完了規約

- 承認済み実装範囲 内の成果だけが返却されている。
- 検証、未実行項目、残留リスク が 根拠参照 付きで整理されている。
- 事前の reproduction condition と同じ条件で観測した。
- 変更前後の差を 観測済み事実 として返した。
- remaining_gaps と residual_risks を分けた。

## 停止規約

- 初回再現を行う時
- 実装中 trace が必要な時
- レビュー の判定だけを行う時
- 単一調査入力、再観測対象、対象調査範囲、検証コマンドが不足する時
- 停止時は不足項目、衝突箇所、戻し先を返す。
