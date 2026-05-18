---
name: implement-integration
description: Codex implementation レーン 側の API / Wails / DTO / gateway など frontend と backend の統合境界実装作業プロトコル。
---
# Implement Integration

## 目的

この skill は作業プロトコルである。
`integration_implementer` agent が API、Wails 紐づけ、DTO、gateway、adapter 契約 など frontend と backend の統合境界 承認済み実装範囲 を実装し、実画面で接続結果を確認する時の判断基準を提供する。

integration は広い frontend / backend 同時変更の許可ではない。
片側だけで閉じる UI 実装や backend 実装は、それぞれ `implement-frontend` または `implement-backend` を使う。

## 対応ロール

- `integration_implementer` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` とする。
- 担当成果物は `implement-integration` の出力規約で固定する。

## 入力規約

- 単一引き継ぎ入力: `implementation-scope` から切り出された統合境界実装用 引き継ぎ 1 件。
- 実行中タスク成果物場所: 実装結果、検証結果、停止理由を書き戻す作業計画フォルダまたは run 成果物フォルダ。
- 実装対象: 変更してよい統合境界のファイル、symbol、公開接点。
- 対象変更範囲: 実装してよい統合境界のプロダクトコード範囲。
- 依存完了情報: 着手前に完了している必要がある依存対象の完了結果。
- 検証コマンド: 実行を許可された backend-local または frontend-local の harness command。
- secret 境界情報: 統合境界で扱う参照値、secret 本体、secret 解決責務層、出力禁止値。
- UI 確認根拠: UI がある task で参照する承認済み `ui-design.md` と確認対象画面。
- 合意済みfrontend保護: UI がある task で承認済み frontend 実装を保護する変更禁止範囲。

## 外部参照規約

- エージェント実行定義と実行境界は [integration_implementer.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/integration_implementer.toml) に従う。
- frontend コーディング規約: frontend 変更がある場合は [coding-guidelines-frontend.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/coding-guidelines-frontend.md) とする。
- backend コーディング規約: backend 変更がある場合は [coding-guidelines-backend.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/coding-guidelines-backend.md) とする。
- lint 規約: [lint-policy.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/lint-policy.md) とする。
- architecture 規約: [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) の統合境界だけを参照する。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

secret 分離観点表は次を拘束する。

| 観点 | 確認内容 |
| --- | --- |
| 参照値 | UI、DTO、read model に出してよい識別子だけを統合境界へ渡す |
| secret 本体 | provider、外部 API、内部認証 に渡す値を参照値と分ける |
| 解決責務 | 参照値から secret 本体を解決する層を 1 つに固定する |
| 出力禁止 | URL、DTO、UI、error summary、構造化 log、audit、要求捕捉へ secret 本体を出さない |
| 検証補助 | 偽 secret 保管先、偽送信経路、要求捕捉を使い、出力禁止先への漏れがないことを確認する |

## 判断規約

- implementation-scope の 承認済み実装範囲 を守る
- integration の対象を API、Wails 紐づけ、DTO、gateway、adapter 契約 の統合境界だけに限定する
- 片側だけで閉じない理由を 対象範囲 成果物 で確認する
- 単一引き継ぎ入力 と 承認済み実装範囲 を確認して プロダクトコード だけを変更する
- 合意済みfrontend保護 がある場合は、承認済み統合境界ファイルだけを frontend 側の変更対象にする
- 合意済みfrontend保護 がある場合は、画面、部品、文言、style を変更しない
- secret を扱う場合は、参照値、secret 本体、secret 解決責務層、出力禁止値を単一引き継ぎ入力で確認する
- `credential_ref`、`secret_ref`、`api_key`、`token` などの field 名を、参照値と secret 本体の同名値として扱わない
- secret 本体は provider、外部 API、内部認証 へ渡す直前に secret 解決責務層から受け取る
- URL、DTO、UI、error summary、構造化 log、audit、要求捕捉へ secret 本体を出さない
- 検証 は frontend、backend、統合境界 契約 の証跡を分ける

- API / Wails / DTO / gateway / adapter 契約 のどれを統合境界として変更したか 終了処理 に残す
- UI がある task では、実画面で主要操作が backend まで到達することを確認する
- UI がある task では、実レスポンスが loading、empty、error、success の UI 状態へ反映されることを確認する
- UI がある task では、console error と Wails 呼び出し失敗が残っていないことを確認する
- UI がある task では、実画面が承認済み `ui-design.md` の主要区画、導線、状態表示から外れていないことを確認する
- secret を扱った場合は、偽 secret 保管先、偽送信経路、要求捕捉による漏れ確認を 終了処理 に残す
- 両側の touched files を 引き継ぎ と対応づける
- frontend / backend / 統合境界 契約 の レーン内検証 根拠 を分ける
- レーン内検証 コマンド の不足を 残留リスク にする

## 非対象規約

- frontend または backend の片側だけで閉じる変更は扱わない。
- integration を広い frontend / backend 同時変更の口実にしない。
- 承認済み統合境界外の API / Wails / DTO / gateway / adapter 契約変更は扱わない。
- 合意済みfrontend保護 がある場合は、画面、部品、文言、style の変更を扱わない。
- プロダクトテスト、検証データ、スナップショット、test helper は変更しない。
- docs や作業流れ文書は変更しない。
- coverage、harness all、repo-local Sonar issue 判定条件は必須終了処理にしない。

## 出力規約

- 判断結果: 統合境界プロダクトコード実装の完了、未完了、停止の判定を返す。
- 根拠参照: 実装の根拠にした入力、変更箇所、検証結果を返す。
- 不足情報: 実装を完了できない不足項目を返す。
- 次判断材料: `implement_lane` が次を判断できる材料を返す。
- 実装成果物: 単一引き継ぎ入力 の 承認済み実装範囲 に対応する統合境界プロダクトコードだけを返す。
- レーン内検証結果: backend-local と frontend-local の失敗時はその場で直して再実行し、通過結果または未実行理由を変更層別に返す。
- 実画面確認結果: UI がある task で確認した画面、主要操作、UI 状態、console error、Wails 呼び出し失敗、未確認理由を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。

## 完了規約

- 承認済み実装範囲 内の成果だけが返却されている。
- 検証、未実行項目、残留リスク が 根拠参照 付きで整理されている。
- 単一引き継ぎ入力、実装対象、対象変更範囲、依存完了情報、検証コマンドを確認した。
- 合意済みfrontend保護 がある場合は、frontend 側の変更が承認済み統合境界ファイルだけで閉じていることを確認した。
- 合意済みfrontend保護 がある場合は、画面、部品、文言、style を変更していないことを確認した。
- API / Wails / DTO / gateway / adapter 契約 の統合境界 対象範囲 が承認済みであることを確認した。
- secret を扱う場合は、参照値、secret 本体、secret 解決責務層、出力禁止値を確認した。
- secret を扱う場合は、偽 secret 保管先、偽送信経路、要求捕捉を使い、secret 本体が URL、DTO、UI、error summary、構造化 log、audit、要求捕捉に出ないことを確認した。
- 両側の touched files を 引き継ぎ と対応づけた。
- 単一引き継ぎ入力 と レーン内検証 根拠 を分けた。
- UI がある task では、実画面で主要操作が backend まで到達することを確認した。
- UI がある task では、実レスポンスが UI 状態へ反映されることを確認した。
- UI がある task では、console error と Wails 呼び出し失敗が残っていないことを確認した。
- UI がある task では、承認済み `ui-design.md` の主要区画、導線、状態表示との差分を確認した。
- backend 側の変更がある場合は `python3 scripts/harness/run.py --suite backend-local` を実行し、失敗した場合は承認済み実装範囲 内でその場で直して再実行し、通過結果または未実行理由を返した。
- frontend 側の変更がある場合は `python3 scripts/harness/run.py --suite frontend-local` を実行し、失敗した場合は承認済み実装範囲 内でその場で直して再実行し、通過結果または未実行理由を返した。
- backend と frontend の両方を含む場合は両方の局所ハーネスを実行し、失敗した場合は承認済み実装範囲 内でその場で直して再実行し、通過結果または未実行理由を返した。

## 停止規約

- frontend または backend の片側だけで閉じる時
- API / Wails / DTO / gateway / adapter 契約 の統合境界変更がない時
- 横断範囲が未承認の時
- 追加設計で横断 対象範囲 を広げる時
- API 統合境界を変えずに UI と backend を同時に触らない
- 単一引き継ぎ入力、実装対象、対象変更範囲、依存完了情報、検証コマンドが不足する場合は停止する。
- 合意済みfrontend保護 がある task で承認済み統合境界ファイル以外の frontend 変更が必要になる場合は停止する。
- 合意済みfrontend保護 がある task で画面、部品、文言、style の変更が必要になる場合は停止する。
- secret を扱う統合境界で、参照値、secret 本体、secret 解決責務層、出力禁止値が不足する場合は停止する。
- secret 本体を URL、DTO、UI、error summary、構造化 log、audit、要求捕捉へ出す必要がある場合は停止する。
- 偽 secret 保管先、偽送信経路、要求捕捉で secret 漏れを確認できない場合は停止する。
- UI がある task で実画面確認根拠を取得できない場合は停止する。
- 実画面確認で承認済み統合境界外の UI 修正が必要になる場合は停止する。
- プロダクトテスト、検証データ、スナップショット、test helper の変更が必要になる場合は停止する。
- `python3 scripts/harness/run.py --suite backend-local` または `python3 scripts/harness/run.py --suite frontend-local` の失敗原因が承認済み実装範囲 外にある場合は停止する。
- 承認済み実装範囲外へ実装を広げる必要がある場合は停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
