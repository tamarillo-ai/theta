---
name: implement-backend
description: Codex implementation レーン 側の backend 実装作業プロトコル。層 責務、レーン内検証 の判断基準を提供する。
---
# Implement Backend

## 目的

この skill は作業プロトコルである。
`backend_implementer` agent が backend 承認済み実装範囲 を実装する時に、usecase、service、repository、adapter の責務整合と 依存方向 を守る判断基準を提供する。

## 対応ロール

- `backend_implementer` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` とする。
- 担当成果物は `implement-backend` の出力規約で固定する。

## 入力規約

- 単一引き継ぎ入力: `implementation-scope` から切り出された backend 実装用 引き継ぎ 1 件。
- 実行中タスク成果物場所: 実装結果、検証結果、停止理由を書き戻す作業計画フォルダまたは run 成果物フォルダ。
- 実装対象: 変更してよい backend ファイル、symbol、公開接点。
- 対象変更範囲: 実装してよい backend プロダクトコード範囲。
- 依存完了情報: 着手前に完了している必要がある依存対象の完了結果。
- 検証コマンド: 実行を許可された backend-local の harness command。

## 外部参照規約

- エージェント実行定義と実行境界は [backend_implementer.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/backend_implementer.toml) に従う。
- コーディング規約: [coding-guidelines-backend.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/coding-guidelines-backend.md) とする。
- lint 規約: [lint-policy.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/lint-policy.md) とする。
- architecture 規約: [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) の backend 境界だけを参照する。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

## 判断規約

- 層 責務と依存方向を守る
- エラー経路 と 検証 を 承認済み実装範囲 内で閉じる
- 単一引き継ぎ入力 と 承認済み実装範囲 を確認して プロダクトコード だけを変更する
- レーン内検証 結果 または未実行理由を返す
- `lint:backend` の format、vet、static、arch、module で落ちる境界違反を事前に避ける

- usecase / service / repository / adapter の責務を確認する
- [lint-policy.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/lint-policy.md) の backend lint 内訳を確認する
- [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) の backend 依存方向に従い、usecase、service、repository、adapter concrete の境界を跨がない
- usecase から repository concrete、実行定義 concrete、driver API を直接参照しない
- レーン内検証 を実行した場合は結果を 終了処理 に残す

## 非対象規約

- frontend だけの変更、UI check、backend 境界の再設計は扱わない。
- 承認済み実装範囲外の層 refactor は扱わない。
- プロダクトテスト、検証データ、スナップショット、test helper は変更しない。
- docs や作業流れ文書は変更しない。
- coverage、harness all、repo-local Sonar issue 判定条件は必須終了処理にしない。

## 出力規約

- 判断結果: backend プロダクトコード実装の完了、未完了、停止の判定を返す。
- 根拠参照: 実装の根拠にした入力、変更箇所、検証結果を返す。
- 不足情報: 実装を完了できない不足項目を返す。
- 次判断材料: `implement_lane` が次を判断できる材料を返す。
- 実装成果物: 単一引き継ぎ入力 の 承認済み実装範囲 に対応する backend プロダクトコードだけを返す。
- レーン内検証結果: `python3 scripts/harness/run.py --suite backend-local` の失敗時はその場で直して再実行し、通過結果または未実行理由を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。

## 完了規約

- 承認済み実装範囲 内の成果だけが返却されている。
- 検証、未実行項目、残留リスク が 根拠参照 付きで整理されている。
- 単一引き継ぎ入力、実装対象、対象変更範囲、依存完了情報、検証コマンドを確認した。
- 層 責務と 依存方向 を確認した。
- backend lint の format、static、arch、module 観点を確認した。
- 検証 と エラー経路 を 承認済み実装範囲 内で確認した。
- backend 変更として `python3 scripts/harness/run.py --suite backend-local` を実行し、失敗した場合は承認済み実装範囲 内でその場で直して再実行し、通過結果または未実行理由を返した。
- 単一引き継ぎ入力 と レーン内検証 を確認した。

## 停止規約

- frontend だけの変更を実装する時
- UI check を行う時
- backend 境界を設計し直す時
- 単一引き継ぎ入力、実装対象、対象変更範囲、依存完了情報、検証コマンドが不足する場合は停止する。
- controller、usecase、service で concrete 実装を new する必要がある場合は停止する。
- service core から filesystem、Wails 実行定義、DB driver の concrete API を直接呼ぶ必要がある場合は停止する。
- プロダクトテスト、検証データ、スナップショット、test helper の変更が必要になる場合は停止する。
- `python3 scripts/harness/run.py --suite backend-local` の失敗原因が承認済み実装範囲 外にある場合は停止する。
- 承認済み実装範囲外へ実装を広げる必要がある場合は停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
