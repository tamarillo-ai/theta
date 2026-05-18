---
name: tests-scenario
description: Codex implementation レーン 側の シナリオテスト 実装作業プロトコル。
---
# Tests Scenario

## 目的

この skill は作業プロトコルである。
`implementation_scenario_tester` agent が承認済みシナリオ または 軽量変更レーンの `task 枠` を シナリオテスト で証明する時の判断基準を提供する。
主対象は `UI人間操作E2E` と `APIテスト` である。

## 対応ロール

- `implementation_scenario_tester` が使う。
- 呼び出し元は `implement_lane` または `light_change_lane` とする。
- 返却先は `implement_lane` または `light_change_lane` とする。
- 担当成果物は `tests-scenario` の出力規約で固定する。

## 入力規約

- 単一引き継ぎ入力: `implementation-scope` から切り出された tests-scenario 用 引き継ぎ 1 件、または 軽量変更レーンの `テスト修正証跡` 用 引き継ぎ 1 件。
- 実行中タスク成果物場所: テスト成果、検証結果、停止理由を書き戻す作業計画フォルダまたは run 成果物フォルダ。
- 対象テスト範囲: 変更してよい シナリオテスト と必要最小限の テスト補助 の path。
- 証明対象: シナリオ ID、実行テスト種別、入力開始点、主要観測点、期待結果。
- 検証コマンド: 実行を許可された backend-local または frontend-local の harness command。

## 外部参照規約

- エージェント実行定義と実行境界は [implementation_scenario_tester.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/implementation_scenario_tester.toml) に従う。
- テストコーディング規約: [coding-guidelines-tests.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/coding-guidelines-tests.md) とする。
- lint 規約: [lint-policy.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/lint-policy.md) とする。
- architecture 規約: 引き継ぎに architecture constraint がある場合だけ [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) を参照する。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

- `UI人間操作E2E`: 承認済みシナリオの開始操作と入力模倣方針を証明する。
- `APIテスト`: 公開接点、要求 / 応答契約、外部入力開始、主要観測点を証明する。
- `実装後シナリオテスト`: 実装済み範囲が承認済みシナリオの結果を満たすことを証明する。

## 判断規約

- 各テストは 1 つの シナリオ 結果 だけを証明する
- setup は決定的にする
- テスト本体に条件分岐を入れない
- 実行定義 event 完了は 完了 event を観測点にする
- `UI人間操作E2E` は、承認済みシナリオの開始操作を模倣する
- UI が入口のシナリオでは、画面操作、ファイル選択、フォーム入力などのユーザー入力を開始点にする
- `APIテスト` は、公開接点、要求 / 応答契約、外部入力開始、主要観測点を開始点にする
- 裏側の直接呼び出しや 検証データ 直接投入だけの試験は、明示された補助試験でない限り主 `UI人間操作E2E` にしない
- 承認済みシナリオと実装済み範囲を元に期待どおり失敗するテストは文脈不足にしない
- 原因未確定の 回帰テスト は実装前に書かない
- coverage、harness all、repo-local Sonar issue 判定条件 は 最終検証 レーン へ defer する

- Arrange / Act / Assert が body 構造で読めるようにする
- 成功経路 と 失敗経路 を別 テストケース に分ける
- 検証データ や補助は シナリオ を支える範囲に限定する
- UI が入口の場合は、ユーザー入力から得られる値を `UI人間操作E2E` の検証対象にする
- `APIテスト` では 要求 / 応答契約 と external 入力 start を検証対象にする

## 非対象規約

- 単体分岐だけの補強、未承認シナリオ、原因未確定の回帰テストは扱わない。
- プロダクトコード修正、新しい要件解釈、paid real AI API 呼び出しは扱わない。
- UI 入口の `UI人間操作E2E` を裏側の直接呼び出しだけで代替しない。

## 出力規約

- 判断結果: シナリオテストを実装したか、文脈不足で停止したかを返す。
- 根拠参照: 単一引き継ぎ入力、証明対象、変更ファイルを返す。
- 不足情報: 不足した入力項目、衝突した根拠、戻し先を返す。
- テスト成果物: 承認済みシナリオに対応する シナリオテスト と必要最小限の 検証データ / 補助 だけを返す。
- 証明済み完了条件: テストで証明した シナリオ結果、公開接点、入力開始点、主要観測点、検証コマンドを返す。
- 未証明小範囲: 同じ 引き継ぎ 内で未証明のシナリオ結果を返す。
- レーン内検証結果: テスト追加または更新後、変更層 に対応する 局所検証 の失敗時はその場で直して再実行し、通過結果または未実行理由を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。

## 完了規約

- 承認済み実装範囲 または 軽量変更レーンの `task 枠` 内の成果だけが返却されている。
- 検証、未実行項目、残留リスク が 根拠参照 付きで整理されている。
- シナリオ 結果 を 1 テスト 1 結果 に分けた。
- `UI人間操作E2E` は、ユーザー入力の模倣を開始点にした。
- `APIテスト` は、公開接点 と外部入力開始を開始点にした。
- 成功経路 と 失敗経路 を別 テストケース にした。
- 実行定義 event 完了の観測点を明示した。
- 変更対象が シナリオテスト と必要最小限の テスト補助だけである。
- backend 側のシナリオテストを変更した場合は `python3 scripts/harness/run.py --suite backend-local` を実行し、失敗した場合は承認済み実装範囲 または 軽量変更レーンの `task 枠` 内でその場で直して再実行し、通過結果または未実行理由を返した。
- frontend 側のシナリオテストを変更した場合は `python3 scripts/harness/run.py --suite frontend-local` を実行し、失敗した場合は承認済み実装範囲 または 軽量変更レーンの `task 枠` 内でその場で直して再実行し、通過結果または未実行理由を返した。
- backend と frontend の両方を含む場合は両方の局所ハーネスを実行し、失敗した場合は承認済み実装範囲 または 軽量変更レーンの `task 枠` 内でその場で直して再実行し、通過結果または未実行理由を返した。
- レーン内検証 の失敗時はその場で直して再実行し、通過結果または未実行理由を返した。

## 停止規約

- 単体 分岐 だけを補う時
- 単一引き継ぎ入力が承認済み implementation-scope または 軽量変更レーンの `task 枠` 由来ではない時
- `python3 scripts/harness/run.py --suite backend-local` または `python3 scripts/harness/run.py --suite frontend-local` の失敗原因が承認済み実装範囲 または 軽量変更レーンの `task 枠` 外にある場合は停止する。
- 原因未確定の 回帰テスト を書く時
- プロダクトコードの修正が主目的の時
- シナリオ結果、公開接点、入力開始点、主要観測点のいずれかが不足している時
- 停止時は不足項目、衝突箇所、戻し先を返す。
- テスト本体に条件分岐が必要になる場合は停止する。
