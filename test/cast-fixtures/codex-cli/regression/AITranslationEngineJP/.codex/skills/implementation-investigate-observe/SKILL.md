---
name: implementation-investigate-observe
description: Codex implementation レーン 側の一時観測点作業プロトコル。
---
# Implementation Investigate Observe

## 目的

この skill は作業プロトコルである。
`implementation_investigator` agent が 承認済み実装範囲 内に一時観測点を add / remove する時の判断基準を提供する。

## 対応ロール

- `implementation_investigator` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` とする。
- 担当成果物は `implementation-investigate-observe` の出力規約で固定する。

## 入力規約

- 単一調査入力: 実装時調査用 引き継ぎから切り出された一時観測作業 1 件。
- 実行中タスク成果物場所: 観測結果、停止理由を書き戻す作業計画フォルダまたは run 成果物フォルダ。
- 観測対象: 一時観測点を置くファイル、symbol、処理経路。
- 対象調査範囲: 一時観測点を追加または削除してよいファイルと symbol。
- 検証コマンド: 観測結果を確認する実行許可済み command。

## 外部参照規約

- エージェント実行定義と実行境界は [implementation_investigator.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/implementation_investigator.toml) に従う。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

## 判断規約

- 一時観測点は 承認済み実装範囲 内に限る
- 観測目的を明確にする
- 返却前に必ず除去する
- cleanup_status を必ず返す

- temporary_changes に path と目的を残す
- cleanup の 検証 を行う
- 除去不能なら stop する

## 非対象規約

- 恒久修正とプロダクトテスト追加は扱わない。
- 承認済み実装範囲外の変更は扱わない。
- cleanup 不能な観測変更は残さない。

## 出力規約

- 判断結果: 一時観測作業の完了、未完了、停止の判定を返す。
- 根拠参照: 一時観測の根拠にした入力、観測点、検証結果を返す。
- 不足情報: 一時観測を完了できない不足項目を返す。
- 次判断材料: 次 agent が判断できる材料を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコードの変更義務を含めない。

## 完了規約

- 承認済み実装範囲 内の成果だけが返却されている。
- 検証、未実行項目、残留リスク が 根拠参照 付きで整理されている。
- temporary_changes に path と目的を残した。
- 観測点を返却前に除去した。
- cleanup_status を必ず返した。

## 停止規約

- 恒久修正を行う時
- プロダクトテスト を追加する時
- cleanup 不能な観測変更が必要な時
- 単一調査入力、観測対象、対象調査範囲、検証コマンドが不足する時
- 停止時は不足項目、衝突箇所、戻し先を返す。
- cleanup 不能な時に続行が必要な場合は停止する。
