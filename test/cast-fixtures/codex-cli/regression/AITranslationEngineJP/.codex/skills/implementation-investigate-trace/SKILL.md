---
name: implementation-investigate-trace
description: Codex implementation レーン 側の実装中 trace 作業プロトコル。
---
# Implementation Investigate Trace

## 目的

この skill は作業プロトコルである。
`implementation_investigator` agent が実装中の原因候補、観測点、不足情報を整理する時の判断基準を提供する。

## 対応ロール

- `implementation_investigator` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` とする。
- 担当成果物は `implementation-investigate-trace` の出力規約で固定する。

## 入力規約

- 単一調査入力: 実装時調査用 引き継ぎから切り出された trace 作業 1 件。
- 実行中タスク成果物場所: trace 結果、停止理由を書き戻す作業計画フォルダまたは run 成果物フォルダ。
- trace 対象: 追跡するファイル、symbol、公開接点、処理経路。
- 対象調査範囲: trace のために読んでよいファイル、symbol、ログ、成果物。

## 外部参照規約

- エージェント実行定義と実行境界は [implementation_investigator.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/implementation_investigator.toml) に従う。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

## 判断規約

- 観測済み事実と仮説を混ぜない
- trace は 承認済み実装範囲 内に限定する
- 不足情報を remaining_gaps に残す
- 根拠 のない結論を固定しない

- 仮説 に根拠と未確認点を付ける
- observation_points を最小にする
- recommended_next_step を根拠付きで返す

## 非対象規約

- 実装前再現、一時観測点 add / remove、恒久修正は扱わない。
- プロダクトテスト追加は扱わない。
- 根拠のない結論は固定しない。

## 出力規約

- 判断結果: trace 作業の完了、未完了、停止の判定を返す。
- 根拠参照: trace の根拠にした入力、ファイル、処理経路を返す。
- 不足情報: trace を完了できない不足項目を返す。
- 次判断材料: 次 agent が判断できる材料を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコードの変更義務を含めない。

## 完了規約

- 承認済み実装範囲 内の成果だけが返却されている。
- 検証、未実行項目、残留リスク が 根拠参照 付きで整理されている。
- 観測済み事実 と 仮説 を分けた。
- observation_points を最小にした。
- recommended_next_step を根拠付きで返した。

## 停止規約

- 実装前再現だけを行う時
- 一時観測点の add / remove が主目的の時
- 恒久修正が主目的の時
- 単一調査入力、trace 対象、対象調査範囲が不足する時
- 停止時は不足項目、衝突箇所、戻し先を返す。
