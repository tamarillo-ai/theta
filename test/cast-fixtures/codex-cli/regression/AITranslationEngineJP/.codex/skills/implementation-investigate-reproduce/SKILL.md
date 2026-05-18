---
name: implementation-investigate-reproduce
description: Codex implementation レーン 側の実装前再現作業プロトコル。
---
# Implementation Investigate Reproduce

## 目的

この skill は作業プロトコルである。
`implementation_investigator` agent が実装前に再現可否と観測事実を確認する時の判断基準を提供する。

## 対応ロール

- `implementation_investigator` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` とする。
- 担当成果物は `implementation-investigate-reproduce` の出力規約で固定する。

## 入力規約

- 単一調査入力: 実装時調査用 引き継ぎから切り出された再現作業 1 件。
- 実行中タスク成果物場所: 再現結果、停止理由を書き戻す作業計画フォルダまたは run 成果物フォルダ。
- 再現対象: 再現する操作、コマンド、入力、公開接点。
- 対象調査範囲: 再現のために読んでよいファイル、symbol、ログ、成果物。

## 外部参照規約

- エージェント実行定義と実行境界は [implementation_investigator.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/implementation_investigator.toml) に従う。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

## 判断規約

- 再現条件と観測結果を分ける
- 根拠 のない原因断定をしない
- 再現できない場合も条件と不足情報を返す
- 実装や test 追加を混ぜない

- コマンド、入力、期待、実際を残す
- reproduction_status を明確にする
- remaining_gaps を次 action へつなげる

## 非対象規約

- 実装中 trace、一時観測点、修正後再観測は扱わない。
- 恒久修正、実装、test 追加は扱わない。
- design 不足を実装側で補わない。

## 出力規約

- 判断結果: 再現作業の完了、未完了、停止の判定を返す。
- 根拠参照: 再現の根拠にした入力、コマンド、観測結果を返す。
- 不足情報: 再現を完了できない不足項目を返す。
- 次判断材料: 次 agent が判断できる材料を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコードの変更義務を含めない。

## 完了規約

- 承認済み実装範囲 内の成果だけが返却されている。
- 検証、未実行項目、残留リスク が 根拠参照 付きで整理されている。
- コマンド、入力、期待、実際を残した。
- reproduction_status を明確にした。
- 観測済み事実 と 仮説 を分けた。

## 停止規約

- 実装中の原因 trace を行う時
- 一時観測点を入れる時
- 修正後の再観測を行う時
- 単一調査入力、再現対象、対象調査範囲が不足する時
- 停止時は不足項目、衝突箇所、戻し先を返す。
